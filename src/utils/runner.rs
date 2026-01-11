//! Benchmark runner: execution engine and data structures.

use std::time::Duration;

use crate::registry::{AlgorithmRunner, BenchmarkResult};
use crate::utils::bench::{shuffle, time_seed, to_nanos, Measurement};
use crate::utils::cpu_affinity::CpuPinGuard;
use crate::utils::timer::{PinStrategy, TimingConfig};
use crate::utils::tui::{print_algo_info_box, print_results_table, sort_variants};

/// Raw timing data for a single variant (used for CSV export)
pub struct RawTimingData {
    pub algo_name: String,
    pub variant_name: String,
    pub input_size: usize,
    pub avg_nanos: u64,
    pub result_sample: Option<f64>,
}

/// Export timing data to CSV file
pub fn export_csv(path: &str, data: &[RawTimingData]) -> std::io::Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(path)?;

    writeln!(file, "algorithm,variant,compiler,input_size,avg_time_ns,result")?;

    for entry in data {
        let compiler = crate::utils::C_COMPILER_NAME.unwrap_or(
            if entry.variant_name.starts_with("c-") {
                "Unknown"
            } else {
                ""
            },
        );

        writeln!(
            file,
            "{},{},{},{},{},{}",
            entry.algo_name,
            entry.variant_name,
            compiler,
            entry.input_size,
            entry.avg_nanos,
            entry.result_sample.map(|v| v.to_string()).unwrap_or_default()
        )?;
    }

    Ok(())
}

/// Run benchmarks for one or more algorithms with global randomization.
///
/// This is the unified entry point for all benchmarking. It:
/// 1. Collects ALL closures from ALL algorithms for ALL sizes into a flat Vec
/// 2. Generates tasks (closure_idx, run_idx) and shuffles globally
/// 3. Executes with CPU pinning
/// 4. Groups results and displays/exports them
pub fn run_benchmarks(
    algorithms: &[&dyn AlgorithmRunner],
    input_sizes: &[usize],
    runs: usize,
    seed: Option<u64>,
    csv_path: Option<&str>,
    filter_outliers: bool,
) {
    let effective_seed = seed.unwrap_or_else(time_seed);
    let config = TimingConfig {
        runs_per_variant: runs,
        warmup_iterations: 10,
        pin_strategy: PinStrategy::PerExecution,
    };

    print_config_info(seed, effective_seed, filter_outliers, &config);

    // 1. Collect closures
    let mut closures = collect_closures(algorithms, input_sizes);
    if closures.is_empty() {
        println!("  No variants to benchmark.");
        return;
    }

    // 2. Warmup & generate tasks
    warmup_closures(&mut closures, config.warmup_iterations);
    let tasks = generate_shuffled_tasks(closures.len(), config.runs_per_variant, effective_seed);

    // 3. Execute
    let (measurements, result_samples) = execute_tasks(&mut closures, tasks, &config);

    // 4. Process & display results
    let (grouped, raw_data) = group_results(
        closures, measurements, result_samples, algorithms, 
        input_sizes.len(), config.runs_per_variant, filter_outliers
    );

    if let Some(path) = csv_path {
        export_csv_with_message(path, &raw_data);
    }

    display_results(algorithms, input_sizes, &grouped, config.runs_per_variant, filter_outliers);
}

// ============================================================================
// Helper functions for run_benchmarks
// ============================================================================

fn print_config_info(seed: Option<u64>, effective_seed: u64, filter_outliers: bool, config: &TimingConfig) {
    println!("  Seed: {} ({})", effective_seed, if seed.is_some() { "user-provided" } else { "time-based" });
    if filter_outliers {
        println!("  Outlier filtering: enabled (trimming 1% extremes)");
    }
    println!("  Pin strategy: {:?}", config.pin_strategy);
}

fn collect_closures<'a>(
    algorithms: &[&'a dyn AlgorithmRunner],
    input_sizes: &[usize],
) -> ClosureVec<'a> {
    println!("  Collecting benchmark closures...");
    let mut closures = Vec::new();

    for (algo_idx, algo) in algorithms.iter().enumerate() {
        for (size_idx, &input_size) in input_sizes.iter().enumerate() {
            for variant in algo.get_variant_closures(input_size) {
                closures.push((
                    ClosureContext {
                        algo_idx,
                        size_idx,
                        input_size,
                        name: variant.name,
                        description: variant.description,
                    },
                    variant.run,
                ));
            }
        }
    }
    closures
}

fn warmup_closures(closures: &mut ClosureVec, iterations: usize) {
    use std::hint::black_box;
    println!("  Warming up {} variants...", closures.len());
    for (_, closure) in closures.iter_mut() {
        for _ in 0..iterations {
            let _ = black_box(closure());
        }
    }
}

fn generate_shuffled_tasks(num_closures: usize, runs: usize, seed: u64) -> Vec<(usize, usize)> {
    let mut tasks: Vec<(usize, usize)> = (0..num_closures)
        .flat_map(|c| (0..runs).map(move |r| (c, r)))
        .collect();
    shuffle(&mut tasks, seed);
    println!("  Running {} tasks (globally randomized)...", tasks.len());
    tasks
}

fn execute_tasks(
    closures: &mut ClosureVec,
    tasks: Vec<(usize, usize)>,
    config: &TimingConfig,
) -> (Vec<Vec<Measurement>>, Vec<Option<f64>>) {
    let runs = config.runs_per_variant;
    let mut measurements: Vec<Vec<Measurement>> = vec![Vec::with_capacity(runs); closures.len()];
    let mut result_samples: Vec<Option<f64>> = vec![None; closures.len()];

    match config.pin_strategy {
        PinStrategy::Global => execute_with_global_pin(closures, tasks, &mut measurements, &mut result_samples),
        PinStrategy::PerExecution => execute_with_per_call_pin(closures, tasks, &mut measurements, &mut result_samples),
    }
    
    println!("\r  Completed!          ");
    println!();
    (measurements, result_samples)
}

fn group_results(
    closures: ClosureVec,
    mut measurements: Vec<Vec<Measurement>>,
    result_samples: Vec<Option<f64>>,
    algorithms: &[&dyn AlgorithmRunner],
    num_sizes: usize,
    runs: usize,
    filter_outliers: bool,
) -> (Vec<Vec<Vec<BenchmarkResult>>>, Vec<RawTimingData>) {
    let num_algos = algorithms.len();
    let mut grouped: Vec<Vec<Vec<BenchmarkResult>>> = vec![vec![Vec::new(); num_sizes]; num_algos];
    let mut raw_data: Vec<RawTimingData> = Vec::new();

    for (closure_idx, (ctx, _)) in closures.into_iter().enumerate() {
        let timing_values = std::mem::take(&mut measurements[closure_idx]);
        let result_sample = result_samples[closure_idx];

        let result = compute_result(&timing_values, ctx.name, ctx.description, runs, result_sample, filter_outliers);

        raw_data.push(RawTimingData {
            algo_name: algorithms[ctx.algo_idx].name().to_string(),
            variant_name: result.name.clone(),
            input_size: ctx.input_size,
            avg_nanos: result.avg_time.as_nanos() as u64,
            result_sample,
        });

        grouped[ctx.algo_idx][ctx.size_idx].push(result);
    }
    (grouped, raw_data)
}

fn export_csv_with_message(path: &str, data: &[RawTimingData]) {
    match export_csv(path, data) {
        Ok(()) => println!("  Raw data exported to: {}", path),
        Err(e) => eprintln!("  Warning: Failed to export CSV: {}", e),
    }
    println!();
}

fn display_results(
    algorithms: &[&dyn AlgorithmRunner],
    input_sizes: &[usize],
    grouped: &[Vec<Vec<BenchmarkResult>>],
    runs: usize,
    filter_outliers: bool,
) {
    for (algo_idx, algo) in algorithms.iter().enumerate() {
        print_algo_info_box(*algo);

        let sizes_with_results = grouped[algo_idx].iter().filter(|r| !r.is_empty()).count();
        let show_size = sizes_with_results > 1;

        for (size_idx, &input_size) in input_sizes.iter().enumerate() {
            let mut results = grouped[algo_idx][size_idx].clone();
            sort_variants(&mut results);

            if !results.is_empty() {
                print_results_table(&results, input_size, runs, show_size, filter_outliers);
            }
        }
    }
}

/// Compute statistics from measurements (Measurement type varies by feature)
fn compute_result(
    values: &[Measurement],
    name: &'static str,
    description: &'static str,
    iterations: usize,
    result_sample: Option<f64>,
    filter_outliers: bool,
) -> BenchmarkResult {
    if values.is_empty() {
        return BenchmarkResult {
            name: name.to_string(),
            description: description.to_string(),
            avg_time: Duration::ZERO,
            median_time: Duration::ZERO,
            min_time: Duration::ZERO,
            max_time: Duration::ZERO,
            std_dev: Duration::ZERO,
            iterations,
            result_sample,
        };
    }

    // Convert to nanos for statistics
    let mut nanos: Vec<u64> = values.iter().map(|m| to_nanos(*m)).collect();
    nanos.sort();
    
    // Apply outlier filtering if requested (trim 0.5% from each end)
    let trimmed = if filter_outliers && nanos.len() > 10 {
        let trim_count = (nanos.len() as f64 * 0.005).ceil() as usize;
        let start = trim_count.min(nanos.len() / 4);
        let end = nanos.len().saturating_sub(trim_count).max(start + 1);
        &nanos[start..end]
    } else {
        &nanos[..]
    };

    let min_val = trimmed[0];
    let max_val = trimmed[trimmed.len() - 1];
    let median_val = trimmed[trimmed.len() / 2];

    let sum: u64 = trimmed.iter().sum();
    let avg_val = sum / trimmed.len() as u64;

    let avg_f = avg_val as f64;
    let variance: f64 = trimmed
        .iter()
        .map(|&v| {
            let diff = v as f64 - avg_f;
            diff * diff
        })
        .sum::<f64>()
        / (trimmed.len() - 1).max(1) as f64;
    let std_dev_val = variance.sqrt() as u64;

    BenchmarkResult {
        name: name.to_string(),
        description: description.to_string(),
        avg_time: Duration::from_nanos(avg_val),
        median_time: Duration::from_nanos(median_val),
        min_time: Duration::from_nanos(min_val),
        max_time: Duration::from_nanos(max_val),
        std_dev: Duration::from_nanos(std_dev_val),
        iterations,
        result_sample,
    }
}

// ============================================================================
// Execution strategies
// ============================================================================

/// Context for each closure during execution
struct ClosureContext {
    algo_idx: usize,
    size_idx: usize,
    input_size: usize,
    name: &'static str,
    description: &'static str,
}

type ClosureVec<'a> = Vec<(ClosureContext, Box<dyn FnMut() -> (Measurement, Option<f64>) + 'a>)>;

/// Execute all tasks with CPU pinned once for the entire session.
/// Minimal overhead - ideal for short-running benchmarks.
fn execute_with_global_pin(
    closures: &mut ClosureVec,
    tasks: Vec<(usize, usize)>,
    measurements: &mut [Vec<Measurement>],
    result_samples: &mut [Option<f64>],
) {
    let total_tasks = tasks.len();
    let report_interval = (total_tasks / 10).max(1);

    // Pin once for entire execution
    let _pin = CpuPinGuard::new();

    for (completed, (closure_idx, _)) in tasks.into_iter().enumerate() {
        let (_, closure) = &mut closures[closure_idx];

        // Timing happens inside the closure
        let (elapsed_time, result) = closure();

        measurements[closure_idx].push(elapsed_time);
        if result.is_some() {
            result_samples[closure_idx] = result;
        }

        if (completed + 1) % report_interval == 0 {
            let pct = ((completed + 1) * 100) / total_tasks;
            print!("\r  Progress: {}%   ", pct);
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
    }
}

/// Execute all tasks with CPU pinned/unpinned for each call.
/// More accurate for long-running benchmarks that might migrate cores.
fn execute_with_per_call_pin(
    closures: &mut ClosureVec,
    tasks: Vec<(usize, usize)>,
    measurements: &mut [Vec<Measurement>],
    result_samples: &mut [Option<f64>],
) {
    let total_tasks = tasks.len();
    let report_interval = (total_tasks / 10).max(1);

    for (completed, (closure_idx, _)) in tasks.into_iter().enumerate() {
        let (_, closure) = &mut closures[closure_idx];

        // Pin for this execution only
        let _pin = CpuPinGuard::new();

        // Timing happens inside the closure
        let (elapsed_time, result) = closure();

        measurements[closure_idx].push(elapsed_time);
        if result.is_some() {
            result_samples[closure_idx] = result;
        }

        if (completed + 1) % report_interval == 0 {
            let pct = ((completed + 1) * 100) / total_tasks;
            print!("\r  Progress: {}%   ", pct);
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
    }
}

