//! Input domains and accuracy reporting shared by scalar math comparisons.

use super::randomness::InputRng;
use super::runner::Config;
use super::scenario::{size_params, ValidationCase};
use super::validation::{Check, CheckResult};
use std::f32::consts::{FRAC_PI_2, PI};

#[derive(Clone, Copy)]
pub enum MathDomain {
    Angles,
    WideAngles,
    Tangent,
    NearPoles,
    Exponents,
    WideExponents,
    PositiveNormal,
    NearOne,
    Powers,
    WidePowers,
}

pub type MathParam = (MathDomain, usize);

impl MathDomain {
    pub fn label(self) -> &'static str {
        match self {
            Self::Angles => "angles-pi",
            Self::WideAngles => "angles-100",
            Self::Tangent => "principal",
            Self::NearPoles => "near-poles",
            Self::Exponents => "exponents-10",
            Self::WideExponents => "exponents-100",
            Self::PositiveNormal => "positive-normal",
            Self::NearOne => "near-one",
            Self::Powers => "powers-small",
            Self::WidePowers => "powers-wide",
        }
    }

    fn sample(self, rng: &mut InputRng) -> [f32; 2] {
        let uniform = |rng: &mut InputRng, low: f32, high: f32| {
            low + (rng.next_f32_range() + 1.0) * 0.5 * (high - low)
        };
        let a = match self {
            Self::Angles => rng.next_f32_range() * PI,
            Self::WideAngles => rng.next_f32_range() * 100.0,
            Self::Tangent => rng.next_f32_range() * 1.4,
            Self::NearPoles => {
                let distance = uniform(rng, 0.001, 0.05);
                let sign = if rng.next_u32_range(2) == 0 {
                    -1.0
                } else {
                    1.0
                };
                sign * (FRAC_PI_2 - distance)
            }
            Self::Exponents => rng.next_f32_range() * 10.0,
            Self::WideExponents => rng.next_f32_range() * 100.0,
            Self::PositiveNormal => {
                let exponent = 1 + rng.next_u32_range(254);
                f32::from_bits((exponent << 23) | rng.next_u32_range(1 << 23))
            }
            Self::NearOne => uniform(rng, 0.5, 2.0),
            Self::Powers => uniform(rng, 0.125, 8.0),
            Self::WidePowers => uniform(rng, 0.01, 100.0),
        };
        let b = match self {
            Self::Powers => rng.next_f32_range() * 4.0,
            Self::WidePowers => rng.next_f32_range() * 8.0,
            _ => 0.0,
        };
        [a, b]
    }
}

pub fn unary_inputs(&(domain, size): &MathParam, rng: &mut InputRng) -> Vec<f32> {
    (0..size).map(|_| domain.sample(rng)[0]).collect()
}

pub fn binary_inputs(&(domain, size): &MathParam, rng: &mut InputRng) -> Vec<[f32; 2]> {
    (0..size).map(|_| domain.sample(rng)).collect()
}

pub fn params(config: &Config, family: &str, domains: &[MathDomain]) -> Vec<MathParam> {
    domains
        .iter()
        .flat_map(|&domain| {
            size_params(config, family)
                .into_iter()
                .map(move |size| (domain, size))
        })
        .collect()
}

pub fn format_param(&(domain, size): &MathParam) -> String {
    format!("{}/n={size}", domain.label())
}

pub fn validation_cases(domains: &[MathDomain]) -> Vec<ValidationCase<MathParam>> {
    domains
        .iter()
        .map(|&domain| ValidationCase::new(domain.label(), (domain, 65_536)))
        .collect()
}

/// Measure differences from std on finite-output domains, without imposing an
/// accuracy tolerance on libraries that intentionally approximate the function.
pub struct FloatErrorReport;

impl Check<Vec<f32>> for FloatErrorReport {
    fn name(&self) -> &str {
        "std-error-report"
    }

    fn evaluate(&self, actual: &Vec<f32>, reference: Option<&Vec<f32>>) -> CheckResult {
        let Some(reference) = reference else {
            return CheckResult::new(
                false,
                "missing reference",
                "std required",
                "accuracy report",
            );
        };
        if actual.is_empty() || actual.len() != reference.len() {
            return CheckResult::new(
                false,
                format!("{} outputs", actual.len()),
                format!("{} nonempty outputs", reference.len()),
                "matching observations required",
            );
        }
        let mut max_absolute = 0.0f64;
        let mut squared_error = 0.0;
        let mut max_relative = 0.0f64;
        let mut relative_samples = 0;
        let mut zero_mismatches = 0;
        let mut mismatches = 0;
        let mut nonfinite = 0;
        for (&actual, &expected) in actual.iter().zip(reference) {
            mismatches += usize::from(actual != expected);
            if !actual.is_finite() || !expected.is_finite() {
                nonfinite += 1;
                continue;
            }
            let error = (f64::from(actual) - f64::from(expected)).abs();
            max_absolute = max_absolute.max(error);
            squared_error += error * error;
            if expected == 0.0 {
                zero_mismatches += usize::from(actual != expected);
            } else {
                relative_samples += 1;
                max_relative = max_relative.max(error / f64::from(expected).abs());
            }
        }
        if nonfinite > 0 {
            max_absolute = f64::INFINITY;
            squared_error = f64::INFINITY;
            max_relative = f64::INFINITY;
        }
        let rmse = (squared_error / actual.len() as f64).sqrt();
        CheckResult::new(nonfinite == 0,
            format!("max_abs={max_absolute:.8e}, rmse={rmse:.8e}, max_rel={max_relative:.8e}, mismatches={mismatches}/{}, nonfinite={nonfinite}", actual.len()),
            "finite outputs; accuracy metrics are informational",
            "comparison with std; relative error excludes zero references")
            .with_metric("max_abs", max_absolute)
            .with_metric("rmse", rmse)
            .with_metric("max_rel", max_relative)
            .with_metric("mismatches", mismatches)
            .with_metric("samples", actual.len())
            .with_metric("relative_samples", relative_samples)
            .with_metric("zero_mismatches", zero_mismatches)
            .with_metric("nonfinite", nonfinite)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_error_handles_zero_references_and_invalid_outputs() {
        let result = FloatErrorReport.evaluate(&vec![0.5, 3.0], Some(&vec![0.0, 2.0]));
        assert!(result.passed);
        assert_eq!(result.metrics["max_abs"], "1");
        assert_eq!(result.metrics["max_rel"], "0.5");
        assert_eq!(result.metrics["relative_samples"], "1");
        assert_eq!(result.metrics["zero_mismatches"], "1");
        assert!(!FloatErrorReport.evaluate(&vec![], Some(&vec![])).passed);
        assert!(
            !FloatErrorReport
                .evaluate(&vec![1.0], Some(&vec![1.0, 2.0]))
                .passed
        );
        assert!(!FloatErrorReport.evaluate(&vec![1.0], None).passed);
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let result = FloatErrorReport.evaluate(&vec![value], Some(&vec![1.0]));
            assert!(!result.passed);
            assert_eq!(result.metrics["nonfinite"], "1");
            assert_eq!(result.metrics["max_abs"], "inf");
        }
    }

    #[test]
    fn comparison_domains_are_reproducible_and_keep_std_outputs_finite() {
        for domain in [
            MathDomain::Angles,
            MathDomain::WideAngles,
            MathDomain::Tangent,
            MathDomain::NearPoles,
            MathDomain::Exponents,
            MathDomain::WideExponents,
            MathDomain::PositiveNormal,
            MathDomain::NearOne,
            MathDomain::Powers,
            MathDomain::WidePowers,
        ] {
            let inputs = binary_inputs(&(domain, 4096), &mut InputRng::new(42));
            assert_eq!(
                inputs,
                binary_inputs(&(domain, 4096), &mut InputRng::new(42))
            );
            for [a, b] in inputs {
                assert!(a.is_finite() && b.is_finite());
                match domain {
                    MathDomain::NearPoles => {
                        let distance = FRAC_PI_2 - a.abs();
                        assert!((0.00099..=0.05001).contains(&distance));
                        assert!(a.tan().is_finite());
                    }
                    MathDomain::Exponents | MathDomain::WideExponents => {
                        assert!(a.exp2().is_normal())
                    }
                    MathDomain::PositiveNormal | MathDomain::NearOne => {
                        assert!(a > 0.0 && a.ln().is_finite())
                    }
                    MathDomain::Powers | MathDomain::WidePowers => {
                        assert!(a > 0.0 && a.powf(b).is_normal())
                    }
                    _ => assert!(a.sin().is_finite() && a.cos().is_finite() && a.tan().is_finite()),
                }
            }
        }
    }
}
