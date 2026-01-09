//! Benchmark execution utilities.
//!
//! Functions for running benchmarks with randomized execution order.

use crate::registry::{AlgorithmRunner, BenchmarkClosure, BenchmarkResult};
use crate::utils::bench::{calculate_std_dev, shuffle_with_rng, time_seed, SeededRng};
use crate::utils::timer::calculate_median;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

/// A single benchmark task with all data needed for execution
struct Task {
    closure: Rc<RefCell<BenchmarkClosure>>,
    algo_idx: usize,
    size_idx: usize,
    closure_idx: usize, // For storing results
}

/// Results grouped by algorithm and size
pub struct GroupedResults {
    /// Results indexed by [algo_idx][size_idx]
    pub results: Vec<Vec<Vec<BenchmarkResult>>>,
    /// Raw timing data for CSV export
    pub raw_data: Vec<RawTimingData>,
}

/// Raw timing data for a single variant
pub struct RawTimingData {
    pub algo_name: String,
    pub variant_name: String,
    pub size: usize,
    pub timings: Vec<Duration>,
    pub sample: f64,
}

/// Warmup all algorithms at all sizes
fn warmup_algorithms(
    algorithms: &[&dyn AlgorithmRunner],
    sample_sizes: &[usize],
    warmup_iterations: usize,
    seed: u64,
) {
    println!("  Warming up...");
    let mut rng = SeededRng::new(seed);

    for &size in sample_sizes {
        for algo in algorithms {
            algo.warmup(size, warmup_iterations, rng.next_u64());
        }
    }
}

fn list_tasks(
    algorithms: &[&dyn AlgorithmRunner],
    sample_sizes: &[usize],
    iterations: usize,
    seed: u64,
) -> Vec<Task> {
    let mut tasks: Vec<Task> = Vec::new();
    let mut num_closures = 0;

    for (size_idx, &sample_size) in sample_sizes.iter().enumerate() {
        for (algo_idx, algo) in algorithms.iter().enumerate() {
            let algo_seed = seed;

            for closure in algo.get_benchmark_closures(sample_size, algo_seed) {
                let closure_idx = num_closures;
                num_closures += 1;

                let shared_closure = Rc::new(RefCell::new(closure));

                // Add all iterations for this closure
                for _ in 0..iterations {
                    tasks.push(Task {
                        closure: Rc::clone(&shared_closure),
                        algo_idx,
                        size_idx,
                        closure_idx,
                    });
                }
            }
        }
    }
    tasks
}

/// Run multiple algorithms with randomized execution order.
///
/// Returns results grouped by algorithm and size, ready for display.
pub fn run_all_algorithms_randomized(
    algorithms: &[&dyn AlgorithmRunner],
    sample_sizes: &[usize],
    iterations: usize,
    seed: Option<u64>,
) -> GroupedResults {
    println!("  Preparing benchmarks...");

    let actual_seed = seed.unwrap_or_else(time_seed);
    let mut rng = SeededRng::new(actual_seed);

    // Collect all tasks - each iteration of each closure is a separate task
    let mut tasks = list_tasks(algorithms, sample_sizes, iterations, actual_seed);
    let num_closures = tasks.len();

    // Warmup
    warmup_algorithms(algorithms, sample_sizes, 3, actual_seed);

    let total_tasks = tasks.len();

    // Shuffle all tasks
    shuffle_with_rng(&mut tasks, &mut rng);

    println!("  Running {} tasks (seed: {})...", total_tasks, actual_seed);

    // Execute
    let mut timings: Vec<Vec<Duration>> = vec![Vec::with_capacity(iterations); num_closures];
    let mut samples: Vec<f64> = vec![0.0; num_closures];

    let report_interval = (total_tasks / 10).max(1);

    for (completed, task) in tasks.iter().enumerate() {
        let (result, elapsed) = (task.closure.borrow_mut().run)();

        timings[task.closure_idx].push(elapsed);
        samples[task.closure_idx] = result;

        if (completed + 1) % report_interval == 0 {
            let pct = ((completed + 1) * 100) / total_tasks;
            print!("\r  Progress: {}%   ", pct);
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
    }
    println!("\r  Completed!          ");
    println!();

    // Build grouped results - need to extract unique closures
    let num_algos = algorithms.len();
    let num_sizes = sample_sizes.len();
    let mut grouped: Vec<Vec<Vec<BenchmarkResult>>> = vec![vec![Vec::new(); num_sizes]; num_algos];
    let mut raw_data: Vec<RawTimingData> = Vec::new();
    let mut seen_closures: Vec<bool> = vec![false; num_closures];

    for task in &tasks {
        if seen_closures[task.closure_idx] {
            continue;
        }
        seen_closures[task.closure_idx] = true;

        let times = &timings[task.closure_idx];
        if times.is_empty() {
            continue;
        }

        let total: Duration = times.iter().sum();
        let avg = total / times.len() as u32;

        let closure = task.closure.borrow();
        let algo = algorithms[task.algo_idx];
        let size = sample_sizes[task.size_idx];

        grouped[task.algo_idx][task.size_idx].push(BenchmarkResult {
            variant_name: closure.name.to_string(),
            description: closure.description.to_string(),
            avg_time: avg,
            median_time: calculate_median(times),
            min_time: *times.iter().min().unwrap(),
            max_time: *times.iter().max().unwrap(),
            std_dev: calculate_std_dev(times, avg),
            iterations: times.len(),
            result_sample: samples[task.closure_idx],
        });

        raw_data.push(RawTimingData {
            algo_name: algo.name().to_string(),
            variant_name: closure.name.to_string(),
            size,
            timings: times.clone(),
            sample: samples[task.closure_idx],
        });
    }

    GroupedResults {
        results: grouped,
        raw_data,
    }
}

/// Export raw timing data to CSV file
pub fn export_csv(path: &str, data: &[RawTimingData]) -> std::io::Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(path)?;

    writeln!(
        file,
        "algorithm,variant,compiler,size,iteration,time_ns,result"
    )?;

    for entry in data {
        // Retrieve global C compiler name for CSV if applicable (not stored per variant anymore but we can lookup or just use global)
        // Or cleaner: just use empty or global if needed.
        let compiler =
            crate::utils::C_COMPILER_NAME.unwrap_or(if entry.variant_name.starts_with("c-") {
                "Unknown"
            } else {
                ""
            });

        for (iter_idx, timing) in entry.timings.iter().enumerate() {
            writeln!(
                file,
                "{},{},{},{},{},{},{}",
                entry.algo_name,
                entry.variant_name,
                compiler,
                entry.size,
                iter_idx,
                timing.as_nanos(),
                entry.sample
            )?;
        }
    }

    Ok(())
}
