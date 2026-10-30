//! Typed checks run only in validation mode. Policies are ordinary Rust types.
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Debug, Write};

use super::csv::csv_field;

fn preview(value: &impl Debug) -> String {
    struct Limited(String);
    impl Write for Limited {
        fn write_str(&mut self, text: &str) -> fmt::Result {
            let remaining = 240usize.saturating_sub(self.0.len());
            if text.len() > remaining {
                let boundary = text
                    .char_indices()
                    .map(|(i, _)| i)
                    .take_while(|i| *i <= remaining)
                    .last()
                    .unwrap_or(0);
                self.0.push_str(&text[..boundary]);
                self.0.push('…');
                return Err(fmt::Error);
            }
            self.0.push_str(text);
            Ok(())
        }
    }
    let mut buffer = Limited(String::new());
    let _ = write!(&mut buffer, "{value:?}");
    buffer.0
}

pub struct CheckResult {
    pub passed: bool,
    pub measured: String,
    pub threshold: String,
    pub reason: String,
    /// Structured values exported as separate `metric_<name>` CSV columns.
    pub metrics: BTreeMap<&'static str, String>,
}

impl CheckResult {
    pub fn new(
        passed: bool,
        measured: impl Into<String>,
        threshold: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            passed,
            measured: measured.into(),
            threshold: threshold.into(),
            reason: reason.into(),
            metrics: BTreeMap::new(),
        }
    }

    pub fn with_metric(mut self, name: &'static str, value: impl ToString) -> Self {
        self.metrics.insert(name, value.to_string());
        self
    }
}

pub trait Check<T> {
    fn name(&self) -> &str;
    fn needs_reference(&self) -> bool {
        true
    }
    fn evaluate(&self, actual: &T, reference: Option<&T>) -> CheckResult;
}

/// Adapt a typed check to an optional observation, rejecting missing outputs.
pub struct Required<T> {
    pub check: Box<dyn Check<T>>,
}
impl<T> Check<Option<T>> for Required<T> {
    fn name(&self) -> &str {
        self.check.name()
    }
    fn needs_reference(&self) -> bool {
        self.check.needs_reference()
    }
    fn evaluate(&self, actual: &Option<T>, reference: Option<&Option<T>>) -> CheckResult {
        let Some(actual) = actual.as_ref() else {
            return CheckResult::new(
                false,
                "missing",
                "observation required",
                "missing observation",
            );
        };
        self.check
            .evaluate(actual, reference.and_then(Option::as_ref))
    }
}

pub struct Exact;
impl<T: PartialEq + Debug> Check<T> for Exact {
    fn name(&self) -> &str {
        "exact"
    }
    fn evaluate(&self, actual: &T, reference: Option<&T>) -> CheckResult {
        CheckResult::new(
            reference == Some(actual),
            preview(actual),
            reference
                .map(preview)
                .unwrap_or_else(|| "missing reference".into()),
            "outputs must match",
        )
    }
}

pub struct NumericTolerance {
    pub absolute: f64,
    pub relative: f64,
    /// An independent oracle overrides reference comparison.
    pub expected: Option<f64>,
}

impl Check<f64> for NumericTolerance {
    fn name(&self) -> &str {
        if self.expected.is_some() {
            "numeric-oracle"
        } else {
            "numeric-parity"
        }
    }
    fn needs_reference(&self) -> bool {
        self.expected.is_none()
    }
    fn evaluate(&self, actual: &f64, reference: Option<&f64>) -> CheckResult {
        let Some(expected) = self.expected.or(reference.copied()) else {
            return CheckResult::new(
                false,
                actual.to_string(),
                "reference required",
                "missing observation",
            );
        };
        let delta = (actual - expected).abs();
        let limit = self.absolute + self.relative * expected.abs();
        let valid_policy = self.absolute.is_finite()
            && self.relative.is_finite()
            && self.absolute >= 0.0
            && self.relative >= 0.0;
        let passed = valid_policy
            && !actual.is_nan()
            && !expected.is_nan()
            && (actual == &expected
                || (actual.is_finite() && expected.is_finite() && delta <= limit));
        CheckResult::new(
            passed,
            format!("actual={actual}, expected={expected}, delta={delta}"),
            format!("abs={} + rel={} * |expected|", self.absolute, self.relative),
            "numeric tolerance",
        )
    }
}

/// Byte-histogram Shannon entropy: a smoke check, not a randomness certification.
pub struct ByteEntropy {
    pub minimum: f64,
}
impl Check<Vec<u64>> for ByteEntropy {
    fn name(&self) -> &str {
        "byte-entropy"
    }
    fn needs_reference(&self) -> bool {
        false
    }
    fn evaluate(&self, actual: &Vec<u64>, _reference: Option<&Vec<u64>>) -> CheckResult {
        let mut bins = [0usize; 256];
        for value in actual {
            for byte in value.to_le_bytes() {
                bins[byte as usize] += 1;
            }
        }
        let count = actual.len() as f64 * 8.0;
        let entropy: f64 = bins
            .into_iter()
            .filter(|n| *n > 0)
            .map(|n| {
                let p = n as f64 / count;
                -p * p.log2()
            })
            .sum();
        CheckResult::new(
            !actual.is_empty()
                && self.minimum.is_finite()
                && (0.0..=8.0).contains(&self.minimum)
                && entropy >= self.minimum,
            format!("{entropy:.8} bits/byte ({} outputs)", actual.len()),
            format!(">= {} bits/byte", self.minimum),
            "byte histogram entropy",
        )
    }
}

/// Lift a check to part of a structured observation, without running the workload again.
pub struct Project<T, U> {
    pub name: &'static str,
    pub project: fn(&T) -> &U,
    pub check: Box<dyn Check<U>>,
}
impl<T, U> Check<T> for Project<T, U> {
    fn name(&self) -> &str {
        self.name
    }
    fn needs_reference(&self) -> bool {
        self.check.needs_reference()
    }
    fn evaluate(&self, actual: &T, reference: Option<&T>) -> CheckResult {
        self.check
            .evaluate((self.project)(actual), reference.map(self.project))
    }
}

pub struct ValidationRecord {
    pub group: &'static str,
    pub scenario: &'static str,
    pub variant: &'static str,
    pub case: String,
    pub seed: u64,
    pub check: String,
    pub result: CheckResult,
}

#[derive(Default)]
pub struct ValidationReport {
    pub records: Vec<ValidationRecord>,
}
impl ValidationReport {
    /// Export all checks, including failures, with optional structured metrics.
    pub fn export_csv(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let mut file = std::io::BufWriter::new(std::fs::File::create(path)?);
        self.write_csv(&mut file)?;
        std::io::Write::flush(&mut file)
    }

    fn write_csv(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        let metric_names: BTreeSet<_> = self
            .records
            .iter()
            .flat_map(|record| record.result.metrics.keys().copied())
            .collect();
        write!(
            writer,
            "group,scenario,variant,case,fixture_seed,check,passed,measured,threshold,reason"
        )?;
        for name in &metric_names {
            write!(writer, ",{}", csv_field(&format!("metric_{name}")))?;
        }
        writeln!(writer)?;
        for record in &self.records {
            write!(
                writer,
                "{},{},{},{},{},{},{},{},{},{}",
                csv_field(record.group),
                csv_field(record.scenario),
                csv_field(record.variant),
                csv_field(&record.case),
                record.seed,
                csv_field(&record.check),
                record.result.passed,
                csv_field(&record.result.measured),
                csv_field(&record.result.threshold),
                csv_field(&record.result.reason),
            )?;
            for name in &metric_names {
                let value = record
                    .result
                    .metrics
                    .get(name)
                    .map(String::as_str)
                    .unwrap_or("");
                write!(writer, ",{}", csv_field(value))?;
            }
            writeln!(writer)?;
        }
        Ok(())
    }

    pub fn passed(&self) -> bool {
        !self.records.is_empty() && self.records.iter().all(|r| r.result.passed)
    }
    pub fn print(&self) {
        for r in &self.records {
            println!(
                "{} {}::{} / {} / {} / {} seed={} measured={} threshold={} ({})",
                if r.result.passed { "PASS" } else { "FAIL" },
                r.group,
                r.scenario,
                r.variant,
                r.case,
                r.check,
                r.seed,
                r.result.measured,
                r.result.threshold,
                r.result.reason
            );
        }
        println!(
            "{} checks, {} failures",
            self.records.len(),
            self.records.iter().filter(|r| !r.result.passed).count()
        );
    }
}

pub fn record_checks<T>(
    report: &mut ValidationReport,
    context: (&'static str, &'static str, &'static str, &str, u64),
    checks: &[Box<dyn Check<T>>],
    actual: &T,
    reference: Option<&T>,
) {
    let (group, scenario, variant, case, seed) = context;
    if checks.is_empty() {
        report.records.push(ValidationRecord {
            group,
            scenario,
            variant,
            case: case.into(),
            seed,
            check: "missing-checks".into(),
            result: CheckResult::new(
                false,
                "no checks",
                "at least one check",
                "scenario has no validation policies",
            ),
        });
    }
    for check in checks {
        report.records.push(ValidationRecord {
            group,
            scenario,
            variant,
            case: case.into(),
            seed,
            check: check.name().into(),
            result: check.evaluate(actual, reference),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_preserves_failed_checks_escapes_text_and_aligns_optional_metrics() {
        let record = |result| ValidationRecord {
            group: "math",
            scenario: "Log2",
            variant: "Original",
            case: "positive-normal".into(),
            seed: 42,
            check: "error".into(),
            result,
        };
        let report = ValidationReport {
            records: vec![
                record(
                    CheckResult::new(false, "a,\"b\"\nc", "limit", "reason")
                        .with_metric("rmse", 1.25),
                ),
                record(CheckResult::new(true, "ok", "limit", "reason").with_metric("max_abs", 2)),
            ],
        };
        let mut output = Vec::new();
        report.write_csv(&mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), concat!(
            "group,scenario,variant,case,fixture_seed,check,passed,measured,threshold,reason,metric_max_abs,metric_rmse\n",
            "math,Log2,Original,positive-normal,42,error,false,\"a,\"\"b\"\"\nc\",limit,reason,,1.25\n",
            "math,Log2,Original,positive-normal,42,error,true,ok,limit,reason,2,\n",
        ));
        let mut too_small = [0u8; 1];
        assert!(report.write_csv(&mut too_small[..]).is_err());
    }

    #[test]
    fn tolerance_boundaries_and_nonfinite_values() {
        let check = NumericTolerance {
            absolute: 0.5,
            relative: 0.0,
            expected: None,
        };
        assert!(check.evaluate(&1.5, Some(&1.0)).passed);
        assert!(!check.evaluate(&1.5001, Some(&1.0)).passed);
        assert!(!check.evaluate(&f64::NAN, Some(&f64::NAN)).passed);
        assert!(check.evaluate(&f64::INFINITY, Some(&f64::INFINITY)).passed);
        assert!(
            !check
                .evaluate(&f64::INFINITY, Some(&f64::NEG_INFINITY))
                .passed
        );
        assert!(!check.evaluate(&0.0, None).passed);
        let relative = NumericTolerance {
            absolute: 0.0,
            relative: 0.1,
            expected: Some(10.0),
        };
        assert!(relative.evaluate(&11.0, None).passed);
        assert!(!relative.evaluate(&11.01, None).passed);
    }
    #[test]
    fn exact_and_entropy_detect_broken_outputs() {
        let required = Required {
            check: Box::new(Exact),
        };
        assert!(!required.evaluate(&None::<u64>, Some(&None)).passed);
        assert!(!required.evaluate(&Some(1u64), None).passed);
        assert!(required.evaluate(&Some(1u64), Some(&Some(1))).passed);
        assert!(!Exact.evaluate(&vec![1u64, 2], Some(&vec![1, 3])).passed);
        let entropy = ByteEntropy { minimum: 7.99 };
        assert!(!entropy.evaluate(&vec![0; 65536], None).passed);
        assert!(!entropy.evaluate(&vec![], None).passed);
        let balanced: Vec<_> = (0u64..256).map(|b| b * 0x0101010101010101).collect();
        assert!(entropy.evaluate(&balanced, None).passed);
        assert!(preview(&vec![u64::MAX; 65536]).len() <= 243);
    }
}
