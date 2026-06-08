//! Benchmark runner: execution engine and data structures using generic RunIds.

use serde::Deserialize;
use std::collections::HashMap;
use std::io::Write;
use std::time::Duration;

use crate::registry::{get_run_metadata, init_session, run_scenario};
use crate::utils::scenario::OtherMetric;

#[derive(Clone, Copy, Debug)]
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
    pub variant: &'static str,
    pub context: String,
    pub seed: u64,
    pub tags: &'static [&'static str],
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Config {
    pub sample: Option<u64>,
    pub filter_scenario: Option<Vec<String>>,
    pub filter_variant: Option<Vec<String>>,
    pub filter_tag: Option<Vec<String>>,
    #[serde(flatten)]
    pub custom: Option<toml::Table>,
}

impl Config {
    pub fn get_table(&self, key: &str) -> Option<&toml::Table> {
        self.custom.as_ref()?.get(key)?.as_table()
    }
}

pub struct RawTimingData {
    pub group: String,
    pub scenario: String,
    pub variant: String,
    pub context: String,
    pub avg_nanos: u64,
    pub other_measurement: String,
}

struct VariantStats {
    name: &'static str,
    avg_nanos_f64: f64,
    min_time: Duration,
    max_time: Duration,
    cv: f64,
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

    let total = session.run_ids.len();
    println!("  Running {} tasks (globally randomized)...", total);

    let start = std::time::Instant::now();
    let mut last_pct = 0;

    struct RunResult {
        run_id: RunId,
        elapsed: u64,
        other: OtherMetric,
    }

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

        let (elapsed, other) = run_scenario(&session, run_id);
        results.push(RunResult {
            run_id,
            elapsed,
            other,
        });
    }

    println!("\n  Completed! Analyzing results...");

    // Group results by (group_name, scenario_name, context)
    // We map: (group, scenario, context) -> variant_name -> Vec<(elapsed, other, seed)>
    type GroupKey = (String, String, String);
    let mut grouped: HashMap<GroupKey, HashMap<String, Vec<(u64, OtherMetric, u64)>>> =
        HashMap::new();

    for r in &results {
        let meta = get_run_metadata(&session, r.run_id);
        let key = (
            meta.group.to_string(),
            meta.scenario.to_string(),
            meta.context.clone(),
        );
        grouped
            .entry(key)
            .or_default()
            .entry(meta.variant.to_string())
            .or_default()
            .push((r.elapsed, r.other.clone(), meta.seed));
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
            let a_num = a.2.parse::<usize>().ok();
            let b_num = b.2.parse::<usize>().ok();
            match (a_num, b_num) {
                (Some(an), Some(bn)) => an.cmp(&bn),
                _ => a.2.cmp(&b.2),
            }
        }
    });

    let mut current_group_scenario = None;

    for key in group_keys {
        let (group, scenario, context) = key.clone();

        if current_group_scenario != Some((group.clone(), scenario.clone())) {
            current_group_scenario = Some((group.clone(), scenario.clone()));
            let title = format!("Algorithm: {}::{}", group, scenario);
            let border = "─".repeat(title.len() + 4);
            println!("\n┌{}┐", border);
            println!("│  {}  │", title);
            println!("└{}┘", border);
        }

        let variants_map = &grouped[&key];
        let mut unique_variants: Vec<_> = variants_map.keys().cloned().collect();
        unique_variants.sort_by_key(|v| variant_sort_key(v));

        let mut variant_stats = Vec::new();
        let mut last_other_measurement = OtherMetric::None;

        for var_name in &unique_variants {
            let runs_data = &variants_map[var_name];
            let mut times: Vec<_> = runs_data.iter().map(|(t, _, _)| *t).collect();
            times.sort();

            let trimmed = if filter_outliers && times.len() > 10 {
                let trim_count = (times.len() as f64 * 0.005).ceil() as usize;
                let start = trim_count.min(times.len() / 4);
                let end = times.len().saturating_sub(trim_count).max(start + 1);
                &times[start..end]
            } else {
                &times[..]
            };

            let min_val = trimmed[0];
            let max_val = trimmed[trimmed.len() - 1];
            let sum: u64 = trimmed.iter().sum();
            let avg_nanos_f64 = sum as f64 / trimmed.len() as f64;

            let variance: f64 = trimmed
                .iter()
                .map(|&v| {
                    let diff = v as f64 - avg_nanos_f64;
                    diff * diff
                })
                .sum::<f64>()
                / (trimmed.len() - 1).max(1) as f64;
            let std_dev_ns = variance.sqrt() as u64;

            let cv = if avg_nanos_f64 > 0.0 {
                std_dev_ns as f64 / avg_nanos_f64
            } else {
                0.0
            };

            // Find a static variant name reference from metadata
            let static_var_name = runs_data
                .first()
                .map(|_| {
                    // Just map back to the static string in one of the RunResults
                    let sample_run = results
                        .iter()
                        .find(|r| {
                            let m = get_run_metadata(&session, r.run_id);
                            m.group == group
                                && m.scenario == scenario
                                && m.context == context
                                && m.variant == var_name
                        })
                        .unwrap();
                    get_run_metadata(&session, sample_run.run_id).variant
                })
                .unwrap_or("unknown");

            variant_stats.push(VariantStats {
                name: static_var_name,
                avg_nanos_f64,
                min_time: Duration::from_nanos(min_val),
                max_time: Duration::from_nanos(max_val),
                cv,
            });

            if let Some((_, other, _)) = runs_data.first() {
                last_other_measurement = other.clone();
            }
        }

        // Correctness verification
        let mut all_variants_match = true;
        let mut mismatch_info = None;
        let mut verified_sample = None;

        // Group runs by seed to compare variants
        let mut runs_by_seed: HashMap<u64, Vec<(&str, OtherMetric)>> = HashMap::new();
        for var_name in &unique_variants {
            for (_, other, seed) in &variants_map[var_name] {
                runs_by_seed
                    .entry(*seed)
                    .or_default()
                    .push((var_name.as_str(), other.clone()));
            }
        }

        for (seed, var_vals) in &runs_by_seed {
            if !var_vals.is_empty() {
                let first_val = &var_vals[0].1;
                if verified_sample.is_none() {
                    verified_sample = Some(first_val.clone());
                }
                for val in var_vals {
                    if val.1 != *first_val {
                        all_variants_match = false;
                        mismatch_info = Some((
                            *seed,
                            var_vals[0].0.to_string(),
                            format!("{}", first_val),
                            val.0.to_string(),
                            format!("{}", val.1),
                        ));
                        break;
                    }
                }
            }
            if !all_variants_match {
                break;
            }
        }

        // Print results table
        print_results_table(
            &variant_stats,
            &context,
            config.sample.unwrap_or(10) as usize,
            filter_outliers,
            all_variants_match,
            mismatch_info,
            verified_sample.as_ref(),
        );

        // Save CSV data
        for stats in &variant_stats {
            csv_data.push(RawTimingData {
                group: group.clone(),
                scenario: scenario.clone(),
                variant: stats.name.to_string(),
                context: context.clone(),
                avg_nanos: stats.avg_nanos_f64 as u64,
                other_measurement: format!("{}", last_other_measurement),
            });
        }
    }

    if let Some(path) = csv_path {
        if let Err(e) = export_csv(path, &csv_data) {
            eprintln!("  Warning: Failed to export CSV: {}", e);
        } else {
            println!("  Raw data exported to: {}", path);
        }
    }
}

fn export_csv(path: &str, data: &[RawTimingData]) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    writeln!(
        file,
        "group,scenario,variant,context,avg_time_ns,other_measurement"
    )?;
    for entry in data {
        writeln!(
            file,
            "{},{},{},{},{},{}",
            entry.group,
            entry.scenario,
            entry.variant,
            entry.context,
            entry.avg_nanos,
            entry.other_measurement
        )?;
    }
    Ok(())
}

fn print_results_table(
    results: &[VariantStats],
    context: &str,
    runs: usize,
    filtered: bool,
    verified: bool,
    mismatch: Option<(u64, String, String, String, String)>,
    verified_sample: Option<&OtherMetric>,
) {
    if results.is_empty() {
        return;
    }

    let variant_col_width = 25;
    let table_width = 86;

    let baseline_time = results.first().map(|r| r.avg_nanos_f64).unwrap_or(1.0);
    let filter_note = if filtered { " (filtered)" } else { "" };

    println!(
        "  Context / Size: {} ({} runs{})",
        context, runs, filter_note
    );
    println!("  {}", "─".repeat(table_width));
    println!(
        "  {:<v_width$} {:>12} {:>12} {:>12} {:>10} {:>9}",
        "Variant",
        "Average",
        "Min",
        "Max",
        "Speedup",
        "CV",
        v_width = variant_col_width
    );
    println!("  {}", "─".repeat(table_width));

    for result in results {
        let speedup = baseline_time / result.avg_nanos_f64;

        let time_str = crate::utils::bench::format_measurement_precise(result.avg_nanos_f64);
        let min_str = crate::utils::bench::format_measurement(result.min_time);
        let max_str = crate::utils::bench::format_measurement(result.max_time);

        println!(
            "  {:<v_width$} {:>12} {:>12} {:>12} {:>9.2}x {:>8.2}%",
            result.name,
            time_str,
            min_str,
            max_str,
            speedup,
            result.cv * 100.0,
            v_width = variant_col_width
        );
    }
    println!("  {}", "─".repeat(table_width));

    if verified {
        match verified_sample {
            Some(OtherMetric::None) | None => {
                println!("  Other Measurements: N/A");
            }
            Some(metric) => {
                println!(
                    "  Other Measurements: Verified (All variants match: {})",
                    metric
                );
            }
        }
    } else if let Some((seed, var1, val1, var2, val2)) = mismatch {
        println!("  ⚠️  WARNING: Other measurements mismatch!");
        println!("     Seed: 0x{:X}", seed);
        println!("     - {:<15}: {}", var1, val1);
        println!("     - {:<15}: {}", var2, val2);
    }
    println!();
}
