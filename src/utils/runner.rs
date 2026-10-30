//! Benchmark runner: execution engine and data structures using generic RunIds.

use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::io::Write;

use crate::registry::{
    get_run_metadata, init_session, run_scenario, warmup_scenario, BenchmarkSession,
};
use crate::utils::cpu_affinity::CpuPinGuard;
use crate::utils::randomness::{derive_seed, InputRng};
use crate::utils::statistics::TimingStatistics;
use crate::utils::timer::{PinStrategy, DEFAULT_WARMUP_ITERATIONS};

pub const DEFAULT_SAMPLES: u64 = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunId(pub u128);

impl RunId {
    pub fn new(scenario_idx: u16, variant_idx: u16, param_idx: u32, seed: u64) -> Self {
        let val = (scenario_idx as u128)
            | ((variant_idx as u128) << 16)
            | ((param_idx as u128) << 32)
            | ((seed as u128) << 64);
        Self(val)
    }

    pub fn unpack(self) -> (u16, u16, u32, u64) {
        (
            (self.0 & 0xFFFF) as u16,
            ((self.0 >> 16) & 0xFFFF) as u16,
            ((self.0 >> 32) & 0xFFFFFFFF) as u32,
            ((self.0 >> 64) & 0xFFFF_FFFF_FFFF_FFFF) as u64,
        )
    }
}

pub struct RunMetadata {
    pub group: &'static str,
    pub scenario: &'static str,
    pub scenario_id: &'static str,
    pub variant: &'static str,
    pub context: String,
    pub seed: u64,
    pub tags: &'static [&'static str],
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Config {
    pub sample: Option<u64>,
    pub seed: Option<u64>,
    pub warmup: Option<usize>,
    #[serde(default)]
    pub pin: PinStrategy,
    pub filter_scenario: Option<Vec<String>>,
    pub filter_variant: Option<Vec<String>>,
    pub filter_tag: Option<Vec<String>>,
    #[serde(flatten)]
    pub custom: Option<toml::Table>,
}

/// Warm every selected implementation/parameter pair using independent fixture and schedule seeds.
fn warmup_runs(session: &BenchmarkSession, iterations: usize) -> Vec<RunId> {
    if iterations == 0 {
        return Vec::new();
    }
    let mut selected = BTreeMap::new();
    for &run in &session.run_ids {
        let (scenario, variant, param, _) = run.unpack();
        selected.entry((scenario, variant, param)).or_insert(run);
    }
    let mut runs = Vec::new();
    for ((scenario, variant, param), run) in selected {
        let meta = get_run_metadata(session, run);
        for iteration in 0..iterations {
            let seed = derive_seed(
                session.root_seed,
                &[
                    "warmup",
                    meta.group,
                    meta.scenario_id,
                    &meta.context,
                    &iteration.to_string(),
                ],
            );
            runs.push(RunId::new(scenario, variant, param, seed));
        }
    }
    InputRng::new(derive_seed(session.root_seed, &["warmup", "schedule"])).shuffle(&mut runs);
    runs
}

impl Config {
    pub fn matches_variant(&self, id: &str) -> bool {
        self.filter_variant
            .as_ref()
            .is_none_or(|filters| filters.iter().any(|f| id.contains(f)))
    }
    pub fn matches_scenario(&self, group: &str, id: &str, name: &str, tags: &[&str]) -> bool {
        self.filter_scenario.as_ref().is_none_or(|filters| {
            filters
                .iter()
                .any(|f| group.contains(f) || id.contains(f) || name.contains(f))
        }) && self
            .filter_tag
            .as_ref()
            .is_none_or(|filters| filters.iter().any(|f| tags.contains(&f.as_str())))
    }
    pub fn get_table(&self, key: &str) -> Option<&toml::Table> {
        self.custom.as_ref()?.get(key)?.as_table()
    }
}

pub struct RawTimingData {
    pub group: String,
    pub scenario: String,
    pub variant: String,
    pub context: String,
    pub avg_measurement: f64,
    pub seed: u64,
    pub unit: &'static str,
    pub total_samples: usize,
    pub statistics: TimingStatistics,
}

struct VariantStats {
    name: &'static str,
    total_samples: usize,
    statistics: TimingStatistics,
}

struct RunResult {
    run_id: RunId,
    elapsed: u64,
    included: bool,
}

fn variant_sort_key(name: &str) -> (u8, String) {
    let name_lower = name.to_lowercase();
    if name_lower == "original" {
        (0, String::new())
    } else if name_lower.contains("asm")
        || name_lower.contains("simd")
        || name_lower.contains("avx")
        || name_lower.contains("neon")
    {
        (3, name_lower)
    } else if name_lower.starts_with("c-") || name_lower.starts_with("c_") {
        (2, name_lower)
    } else {
        (1, name_lower)
    }
}

#[allow(clippy::type_complexity)]
pub fn run_benchmarks(config: Config, csv_path: Option<&str>, filter_outliers: bool) {
    let session = init_session(&config);
    if session.run_ids.is_empty() {
        println!("  No scenarios to benchmark.");
        return;
    }

    println!(
        "  Seed: {} (replay with --seed {})",
        session.root_seed, session.root_seed
    );

    let warmups = warmup_runs(&session, config.warmup.unwrap_or(DEFAULT_WARMUP_ITERATIONS));
    let initial_pin = CpuPinGuard::new();
    let core = initial_pin.core_id();
    if let Some(core) = core {
        println!("  CPU pinning: core {core} ({:?})", config.pin);
    } else {
        eprintln!("  CPU pinning unavailable; continuing without affinity.");
    }
    let global_pin = match config.pin {
        PinStrategy::Global => Some(initial_pin),
        PinStrategy::PerExecution => {
            drop(initial_pin);
            None
        }
    };
    let mut pin_failed = false;
    let mut pin_execution = || {
        let guard = (config.pin == PinStrategy::PerExecution)
            .then(|| core.map(CpuPinGuard::with_core))
            .flatten();
        if guard.as_ref().is_some_and(|guard| !guard.is_pinned()) && !pin_failed {
            eprintln!(
                "  CPU pinning failed for the selected core; some executions may run unpinned."
            );
            pin_failed = true;
        }
        guard
    };
    println!(
        "  Warming up {} tasks (excluded from results)...",
        warmups.len()
    );
    for run in warmups {
        let _pin = pin_execution();
        warmup_scenario(&session, run);
    }

    let total = session.run_ids.len();
    println!("  Running {} tasks (globally randomized)...", total);

    let start = std::time::Instant::now();
    let mut last_pct = 0;

    let mut results = Vec::with_capacity(total);

    for (i, &run_id) in session.run_ids.iter().enumerate() {
        let pct = (i + 1) * 100 / total;
        if pct >= last_pct + 10 || i + 1 == total {
            last_pct = pct;
            let elapsed_sec = start.elapsed().as_secs_f64();
            let eta = elapsed_sec / (i + 1) as f64 * (total - i - 1) as f64;
            eprintln!(
                "[{pct:3}%] {}/{} elapsed: {:.1}s ETA: {:.1}s",
                i + 1,
                total,
                elapsed_sec,
                eta
            );
        }

        let elapsed = {
            let _pin = pin_execution();
            run_scenario(&session, run_id)
        };
        results.push(RunResult {
            run_id,
            elapsed,
            included: true,
        });
    }
    drop(global_pin);

    println!("\n  Completed! Analyzing results...");

    // Group timing samples; validation is an independent execution path.
    type GroupKey = (String, String, String, String);
    let mut grouped: HashMap<GroupKey, HashMap<&'static str, Vec<usize>>> = HashMap::new();

    for (index, r) in results.iter().enumerate() {
        let meta = get_run_metadata(&session, r.run_id);
        let key = (
            meta.group.to_string(),
            meta.scenario_id.to_string(),
            meta.scenario.to_string(),
            meta.context.clone(),
        );
        grouped
            .entry(key)
            .or_default()
            .entry(meta.variant)
            .or_default()
            .push(index);
    }

    let mut csv_data: Vec<RawTimingData> = Vec::new();

    // Sort group keys for stable display
    let mut group_keys: Vec<_> = grouped.keys().cloned().collect();
    group_keys.sort_by(|a, b| {
        if a.0 != b.0 {
            a.0.cmp(&b.0)
        } else if a.1 != b.1 {
            a.1.cmp(&b.1)
        } else {
            let a_num = a.3.parse::<usize>().ok();
            let b_num = b.3.parse::<usize>().ok();
            match (a_num, b_num) {
                (Some(an), Some(bn)) => an.cmp(&bn),
                _ => a.3.cmp(&b.3),
            }
        }
    });

    let mut current_group_scenario = None;

    for key in group_keys {
        let (group, scenario_id, scenario, context) = key.clone();

        if current_group_scenario != Some((group.clone(), scenario_id.clone())) {
            current_group_scenario = Some((group.clone(), scenario_id.clone()));
            let title = format!("Algorithm: {}::{} [{}]", group, scenario, scenario_id);
            let border = "─".repeat(title.len() + 4);
            println!("\n┌{}┐", border);
            println!("│  {}  │", title);
            println!("└{}┘", border);
        }

        let variants_map = &grouped[&key];
        let mut unique_variants: Vec<_> = variants_map.keys().cloned().collect();
        unique_variants.sort_by_key(|v| variant_sort_key(v));

        let mut variant_stats = Vec::new();

        for var_name in &unique_variants {
            let mut indices = variants_map[var_name].clone();
            indices.sort_by_key(|&index| (results[index].elapsed, index));

            let retained = if filter_outliers && indices.len() > 10 {
                let trim_count = (indices.len() as f64 * 0.005).ceil() as usize;
                let start = trim_count.min(indices.len() / 4);
                let end = indices.len().saturating_sub(trim_count).max(start + 1);
                for &index in indices[..start].iter().chain(&indices[end..]) {
                    results[index].included = false;
                }
                &indices[start..end]
            } else {
                &indices[..]
            };
            let times: Vec<_> = retained
                .iter()
                .map(|&index| results[index].elapsed)
                .collect();
            variant_stats.push(VariantStats {
                name: var_name,
                total_samples: indices.len(),
                statistics: TimingStatistics::new(&times).expect("nonempty timing samples"),
            });
        }
        print_results_table(
            &variant_stats,
            &context,
            config.sample.unwrap_or(DEFAULT_SAMPLES) as usize,
            filter_outliers,
        );

        // Save CSV data
        for stats in variant_stats {
            csv_data.push(RawTimingData {
                group: group.clone(),
                scenario: scenario_id.clone(),
                variant: stats.name.to_string(),
                context: context.clone(),
                avg_measurement: stats.statistics.mean,
                seed: session.root_seed,
                unit: crate::utils::bench::unit_name(),
                total_samples: stats.total_samples,
                statistics: stats.statistics,
            });
        }
    }

    if let Some(path) = csv_path {
        if let Err(e) = export_csv(path, &csv_data) {
            eprintln!("  Warning: Failed to export CSV: {}", e);
        } else {
            println!("  Statistics exported to: {}", path);
        }
        let samples_path = std::path::Path::new(path).with_extension("samples.csv");
        if let Err(error) = export_samples(&samples_path, &session, &results) {
            eprintln!("  Warning: Failed to export samples: {error}");
        } else {
            println!(
                "  Individual samples exported to: {}",
                samples_path.display()
            );
        }
    }
}

fn export_csv(path: &str, data: &[RawTimingData]) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    writeln!(
        file,
        "group,scenario,variant,context,avg_measurement,seed,unit,samples,total_samples,median_measurement,p95_measurement,min_measurement,max_measurement,stddev_measurement,cv,ci95_low,ci95_high"
    )?;
    for entry in data {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            csv_field(&entry.group),
            csv_field(&entry.scenario),
            csv_field(&entry.variant),
            csv_field(&entry.context),
            entry.avg_measurement,
            entry.seed,
            entry.unit,
            entry.statistics.samples,
            entry.total_samples,
            entry.statistics.median,
            entry.statistics.p95,
            entry.statistics.min,
            entry.statistics.max,
            optional_number(entry.statistics.stddev),
            optional_number(entry.statistics.cv),
            optional_number(entry.statistics.mean_ci95.map(|(low, _)| low)),
            optional_number(entry.statistics.mean_ci95.map(|(_, high)| high))
        )?;
    }
    Ok(())
}

fn optional_number(value: Option<f64>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn export_samples(
    path: &std::path::Path,
    session: &BenchmarkSession,
    results: &[RunResult],
) -> std::io::Result<()> {
    let mut file = std::io::BufWriter::new(std::fs::File::create(path)?);
    writeln!(file, "group,scenario,variant,context,measurement,seed,fixture_seed,unit,execution_index,included")?;
    for (index, result) in results.iter().enumerate() {
        let meta = get_run_metadata(session, result.run_id);
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{}",
            csv_field(meta.group),
            csv_field(meta.scenario_id),
            csv_field(meta.variant),
            csv_field(&meta.context),
            result.elapsed,
            session.root_seed,
            meta.seed,
            crate::utils::bench::unit_name(),
            index,
            result.included
        )?;
    }
    file.flush()
}

fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn print_results_table(results: &[VariantStats], context: &str, runs: usize, filtered: bool) {
    if results.is_empty() {
        return;
    }

    let variant_col_width = 25;
    let table_width = 128;

    let baseline_time = results[0].statistics.mean;
    let filter_note = if filtered { " (filtered)" } else { "" };

    println!(
        "  Context / Size: {} ({} runs{})",
        context, runs, filter_note
    );
    println!("  {}", "─".repeat(table_width));
    println!(
        "  {:<v_width$} {:>5} {:>14} {:>14} {:>14} {:>14} {:>10} {:>9}",
        "Variant",
        "N",
        "Average",
        "Median",
        "P95",
        "95% CI +/-",
        "Speedup",
        "CV",
        v_width = variant_col_width
    );
    println!("  {}", "─".repeat(table_width));

    for result in results {
        let stats = &result.statistics;
        let speedup = if baseline_time > 0.0 && stats.mean > 0.0 {
            format!("{:.2}x", baseline_time / stats.mean)
        } else {
            "N/A".into()
        };
        let format = crate::utils::bench::format_measurement_precise;
        let confidence = stats
            .mean_ci95
            .map(|(low, high)| format((high - low) / 2.0))
            .unwrap_or_else(|| "N/A".into());
        let cv = stats
            .cv
            .map(|cv| format!("{:.2}%", cv * 100.0))
            .unwrap_or_else(|| "N/A".into());

        println!(
            "  {:<v_width$} {:>5} {:>14} {:>14} {:>14} {:>14} {:>10} {:>9}",
            result.name,
            stats.samples,
            format(stats.mean),
            format(stats.median),
            format(stats.p95),
            confidence,
            speedup,
            cv,
            v_width = variant_col_width
        );
    }
    println!("  {}", "─".repeat(table_width));
    println!("  CI is an approximate interval for the mean; speedups are descriptive, not significance tests.");

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warmup_preserves_measured_schedule_and_has_its_own_replayable_fixtures() {
        let config = Config {
            seed: Some(42),
            sample: Some(1),
            filter_scenario: Some(vec!["dot_product".into()]),
            ..Default::default()
        };
        let session = init_session(&config);
        let measured = session.run_ids.clone();
        let warmups = warmup_runs(&session, 3);
        assert_eq!(warmups.len(), 3 * measured.len());
        assert_eq!(warmups, warmup_runs(&session, 3));
        assert_eq!(measured, session.run_ids);

        let more_samples = init_session(&Config {
            sample: Some(5),
            warmup: Some(100),
            pin: PinStrategy::Global,
            ..config.clone()
        });
        assert_eq!(warmups, warmup_runs(&more_samples, 3));
        let settings_only = init_session(&Config {
            warmup: Some(0),
            pin: PinStrategy::Global,
            ..config
        });
        assert_eq!(measured, settings_only.run_ids);

        let measured_seeds: std::collections::BTreeSet<_> =
            measured.iter().map(|run| run.unpack().3).collect();
        let mut fixtures: BTreeMap<_, BTreeMap<_, Vec<_>>> = BTreeMap::new();
        for run in warmups {
            let meta = get_run_metadata(&session, run);
            assert!(!measured_seeds.contains(&meta.seed));
            fixtures
                .entry((meta.group, meta.scenario_id, meta.context))
                .or_default()
                .entry(meta.variant)
                .or_default()
                .push(meta.seed);
        }
        for variants in fixtures.values_mut() {
            for seeds in variants.values_mut() {
                seeds.sort();
                assert_eq!(seeds.len(), 3);
            }
            let expected = variants.values().next().unwrap();
            assert!(variants.values().all(|seeds| seeds == expected));
        }
        assert!(warmup_runs(&session, 0).is_empty());
    }
}
