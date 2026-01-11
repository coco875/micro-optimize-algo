//! Text User Interface (TUI) utilities.
//!
//! Handles formatted output for the CLI.

use crate::registry::{AlgorithmRegistry, AlgorithmRunner, BenchmarkResult};
use crate::utils::runner;
use terminal_size::{terminal_size, Width};

/// Get the current terminal width, defaulting to 100 if detection fails
/// Get the current terminal width, constrained to a reasonable range
fn get_term_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        // Clamp width to avoid layout issues on very small or very large terminals
        (w as usize).clamp(40, 200)
    } else {
        80 // Safe default
    }
}

/// Get sorting priority for a variant based on its name and compiler.
/// Lower values sort first.
/// Order: original (0), Rust (1), C by compiler then name (2), ASM (3)
fn variant_sort_key(result: &BenchmarkResult) -> (u8, String, String) {
    let name = result.name.to_lowercase();
    let compiler = if name.starts_with("c-") || name.starts_with("c_") {
        crate::utils::C_COMPILER_NAME
            .unwrap_or("unknown")
            .to_lowercase()
    } else {
        String::new()
    };

    if name == "original" {
        (0, String::new(), String::new())
    } else if name.contains("asm")
        || name.contains("simd")
        || name.contains("avx")
        || name.contains("neon")
    {
        // ASM/SIMD variants
        (3, name.clone(), compiler)
    } else if name.starts_with("c-") || name.starts_with("c_") {
        // C variants (have compiler or c-/c_ prefix)
        (2, compiler.clone(), name.clone())
    } else {
        // Rust variants (no compiler, no c- prefix)
        (1, name.clone(), String::new())
    }
}

/// Sort variants: original first, then grouped by language (Rust, C, ASM)
fn sort_variants(results: &mut [BenchmarkResult]) {
    results.sort_by_key(variant_sort_key);
}

/// Print algorithm info box
pub fn print_algo_info_box(algo: &dyn AlgorithmRunner) {
    let term_width = get_term_width();
    let max_content_width = term_width.saturating_sub(4).max(40); // Minimal width of 40

    let variants_str = algo.available_variants().join(", ");
    let name_line = format!("Algorithm: {}", algo.name());
    let cat_line = format!("Category:  {}", algo.category());
    let desc_line = algo.description();
    let var_line = format!("Variants: {}", variants_str);

    // Calculate required width based on content, capped at terminal width
    let content_width = [
        name_line.len(),
        cat_line.len(),
        desc_line.len(),
        var_line.len(),
    ]
    .iter()
    .cloned()
    .max()
    .unwrap_or(60)
    .min(max_content_width);

    let border = "─".repeat(content_width + 2);

    println!("┌{}┐", border);
    println!(
        "│ {:<width$} │",
        truncate(&name_line, content_width),
        width = content_width
    );
    println!(
        "│ {:<width$} │",
        truncate(&cat_line, content_width),
        width = content_width
    );
    println!(
        "│ {:<width$} │",
        truncate(desc_line, content_width),
        width = content_width
    );
    println!("├{}┤", border);
    println!(
        "│ {:<width$} │",
        truncate(&var_line, content_width),
        width = content_width
    );
    println!("└{}┘", border);
    println!();
}

/// Truncate string with ellipsis if it exceeds width (character-wise)
fn truncate(s: &str, width: usize) -> String {
    if s.chars().count() <= width {
        s.to_string()
    } else {
        let mut result: String = s.chars().take(width.saturating_sub(3)).collect();
        result.push_str("...");
        result
    }
}

/// Print results table for a single size
/// If show_size is false, the "Size: X" header line is omitted
pub fn print_results_table(results: &[BenchmarkResult], size: usize, runs: usize, show_size: bool, filtered: bool) {
    if results.is_empty() {
        return;
    }

    let term_width = get_term_width();
    // Compact columns: 12+12+12+9+9+10 = 64 chars + 6 spaces + 2 indent = 72
    let fixed_width = 72;
    // Calculate variant width based on remaining space, min 15
    let variant_col_width = term_width.saturating_sub(fixed_width).max(15);
    let table_width = variant_col_width + 64 + 6; // variant + columns + spaces

    let baseline_time = results
        .first()
        .map(|r| r.avg_time.as_nanos() as f64)
        .unwrap_or(1.0);

    let baseline_result = results.first().and_then(|r| r.result_sample);

    let filter_note = if filtered { " (filtered)" } else { "" };
    if show_size {
        println!("  Size: {} ({} runs{})", size, runs, filter_note);
    } else if filtered {
        println!("  {} runs{}", runs, filter_note);
    }
    println!("  {}", "─".repeat(table_width));
    println!(
        "  {:>v_width$} {:>12} {:>12} {:>12} {:>9} {:>9} {:>10}",
        "Variant",
        "Average",
        "Min",
        "Max",
        "Speedup",
        "CV",
        "Rel. Error",
        v_width = variant_col_width
    );
    println!("  {}", "─".repeat(table_width));

    for result in results {
        let speedup = baseline_time / result.avg_time.as_nanos() as f64;

        let avg_ns = result.avg_time.as_nanos() as f64;
        let std_dev_ns = result.std_dev.as_nanos() as f64;

        let cv = if avg_ns > 0.0 {
            std_dev_ns / avg_ns
        } else {
            0.0
        };

        // Compute relative error only if both result_sample and baseline are available
        let relative_error = match (result.result_sample, baseline_result) {
            (Some(res), Some(base)) => {
                let diff = (res - base).abs();
                if base.abs() > 1e-9 {
                    diff / base.abs()
                } else {
                    diff
                }
            }
            _ => 0.0, // No relative error for control flow algorithms
        };

        let display_name =
            if result.name.starts_with("c-") || result.name.starts_with("c_") {
                match crate::utils::C_COMPILER_NAME {
                    Some(c) => format!("{} ({})", result.name, c),
                    None => result.name.clone(),
                }
            } else {
                result.name.clone()
            };

        let time_str = crate::utils::bench::format_measurement(result.avg_time);
        let min_str = crate::utils::bench::format_measurement(result.min_time);
        let max_str = crate::utils::bench::format_measurement(result.max_time);

        println!(
            "  {:<v_width$} {:>12} {:>12} {:>12} {:>8.2}x {:>8.2}% {:>10.2e}",
            truncate(&display_name, variant_col_width),
            time_str,
            min_str,
            max_str,
            speedup,
            cv * 100.0,
            relative_error,
            v_width = variant_col_width
        );
    }
    println!();
}

/// Print the application header
pub fn print_header() {
    let term_width = get_term_width().min(80); // Cap header at 80
    let title = " Micro-Optimize-Algo Benchmarks ";
    let padding = term_width.saturating_sub(title.len() + 2) / 2;
    let right_padding = term_width.saturating_sub(padding + title.len());

    let border = "═".repeat(term_width);

    println!("╔{}╗", border);
    println!(
        "║{}{}{}║",
        " ".repeat(padding),
        title,
        " ".repeat(right_padding)
    );
    println!("╚{}╝", border);
    println!();
}

/// Print the help message
pub fn print_help() {
    println!("Usage: micro-algo [OPTIONS] [ALGORITHM]");
    println!();
    println!("Options:");
    println!("  --list, -l     List all available algorithms");
    println!("  --help, -h     Show this help message");
    println!("  --sizes SIZES  Comma-separated vector sizes (default: 64,256,1024,4096,16384)");
    println!("  --iter N, -r   Number of measurement runs per variant (default: 30)");
    println!("  --seed N       Random seed for reproducible benchmarks (default: time-based)");
    println!("  --filter, -f   Filter outliers (trim 1%% extremes from measurements)");
    println!();
    println!("Arguments:");
    println!("  ALGORITHM      Name of specific algorithm to run (omit for all)");
    println!();
    println!("Examples:");
    println!("  micro-algo                    # Run all algorithms");
    println!("  micro-algo dot_product        # Run only dot_product");
    println!("  micro-algo --list             # List algorithms");
    println!("  micro-algo --sizes 128,512    # Custom sizes");
    println!("  micro-algo --seed 12345       # Reproducible run");
    println!("  micro-algo --csv data.csv     # Export raw timings to CSV");
}

/// Print the list of available algorithms
pub fn print_available_algorithms(registry: &AlgorithmRegistry) {
    println!("Available algorithms:");
    println!();
    for algo in registry.all() {
        println!(
            "  {:<20} [{}] - {}",
            algo.name(),
            algo.category(),
            algo.description()
        );
    }
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
    use crate::utils::bench::{shuffle, time_seed, to_nanos, Measurement};
    use crate::utils::cpu_affinity::CpuPinGuard;
    use crate::utils::timer::{PinStrategy, TimingConfig};
    use std::hint::black_box;
    use std::time::Duration;

    // Use user-provided seed or fall back to time-based seed
    let effective_seed = seed.unwrap_or_else(time_seed);

    let num_algos = algorithms.len();
    let num_sizes = input_sizes.len();

    let config = TimingConfig {
        runs_per_variant: runs,
        warmup_iterations: 10,
        pin_strategy: PinStrategy::PerExecution,
    };

    println!("  Seed: {} ({})", effective_seed, if seed.is_some() { "user-provided" } else { "time-based" });
    if filter_outliers {
        println!("  Outlier filtering: enabled (trimming 1% extremes)");
    }
    println!("  Collecting benchmark closures...");

    // Context for each closure
    struct ClosureContext {
        algo_idx: usize,
        size_idx: usize,
        input_size: usize,
        name: &'static str,
        description: &'static str,
    }

    // 1. Collect ALL closures into a flat Vec
    let mut closures: Vec<(ClosureContext, Box<dyn FnMut() -> (Measurement, Option<f64>) + '_>)> = Vec::new();

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

    if closures.is_empty() {
        println!("  No variants to benchmark.");
        return;
    }

    // 2. Warmup
    println!("  Warming up {} variants...", closures.len());
    for (_, closure) in &mut closures {
        for _ in 0..config.warmup_iterations {
            let _ = black_box(closure());
        }
    }

    // 3. Generate and shuffle tasks globally: (closure_idx, run_idx)
    let runs = config.runs_per_variant;
    let mut tasks: Vec<(usize, usize)> = (0..closures.len())
        .flat_map(|c| (0..runs).map(move |r| (c, r)))
        .collect();
    shuffle(&mut tasks, effective_seed);

    let total_tasks = tasks.len();
    println!("  Running {} tasks (globally randomized)...", total_tasks);

    // 4. Execute all tasks with measurements
    let mut measurements: Vec<Vec<Measurement>> = vec![Vec::with_capacity(runs); closures.len()];
    let mut result_samples: Vec<Option<f64>> = vec![None; closures.len()];

    let report_interval = (total_tasks / 10).max(1);

    for (completed, (closure_idx, _)) in tasks.into_iter().enumerate() {
        let (_, closure) = &mut closures[closure_idx];
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
    println!("\r  Completed!          ");
    println!();

    // 5. Group results by algorithm and size
    let mut grouped: Vec<Vec<Vec<BenchmarkResult>>> = vec![vec![Vec::new(); num_sizes]; num_algos];
    let mut raw_data: Vec<runner::RawTimingData> = Vec::new();

    for (closure_idx, (ctx, _)) in closures.into_iter().enumerate() {
        let timing_values = std::mem::take(&mut measurements[closure_idx]);
        let result_sample = result_samples[closure_idx];

        let result = compute_result(&timing_values, ctx.name, ctx.description, runs, result_sample, filter_outliers);

        raw_data.push(runner::RawTimingData {
            algo_name: algorithms[ctx.algo_idx].name().to_string(),
            variant_name: result.name.clone(),
            input_size: ctx.input_size,
            avg_nanos: result.avg_time.as_nanos() as u64,
            result_sample,
        });

        grouped[ctx.algo_idx][ctx.size_idx].push(result);
    }

    // 6. Export CSV if requested
    if let Some(path) = csv_path {
        match runner::export_csv(path, &raw_data) {
            Ok(()) => println!("  Raw data exported to: {}", path),
            Err(e) => eprintln!("  Warning: Failed to export CSV: {}", e),
        }
        println!();
    }

    // 7. Display results grouped by algorithm
    for (algo_idx, algo) in algorithms.iter().enumerate() {
        print_algo_info_box(*algo);

        // Count how many sizes have results for this algorithm
        let sizes_with_results = grouped[algo_idx]
            .iter()
            .filter(|results| !results.is_empty())
            .count();
        let show_size = sizes_with_results > 1;

        for (size_idx, &input_size) in input_sizes.iter().enumerate() {
            let mut results = grouped[algo_idx][size_idx].clone();
            sort_variants(&mut results);

            if !results.is_empty() {
                print_results_table(&results, input_size, runs, show_size, filter_outliers);
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
}


