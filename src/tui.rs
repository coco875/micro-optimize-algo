//! Text User Interface (TUI) utilities.
//!
//! Handles formatted output for the CLI.

use crate::registry::{AlgorithmRegistry, AlgorithmRunner, BenchmarkResult};
use crate::utils::runner;

/// Get sorting priority for a variant based on its name and compiler.
/// Lower values sort first.
/// Order: original (0), Rust (1), C by compiler then name (2), ASM (3)
fn variant_sort_key(result: &BenchmarkResult) -> (u8, String, String) {
    let name = result.variant_name.to_lowercase();
    let compiler = result.compiler.clone().unwrap_or_default().to_lowercase();

    if name == "original" {
        (0, String::new(), String::new())
    } else if name.contains("asm")
        || name.contains("simd")
        || name.contains("avx")
        || name.contains("neon")
    {
        // ASM/SIMD variants
        (3, name.clone(), compiler)
    } else if name.starts_with("c-") || name.starts_with("c_") || !compiler.is_empty() {
        // C variants (have compiler or c-/c_ prefix)
        (2, compiler.clone(), name.clone())
    } else {
        // Rust variants (no compiler, no c- prefix)
        (1, name.clone(), String::new())
    }
}

/// Sort variants: original first, then grouped by language (Rust, C, ASM)
fn sort_variants(results: &mut [BenchmarkResult]) {
    results.sort_by(|a, b| variant_sort_key(a).cmp(&variant_sort_key(b)));
}

/// Print algorithm info box
pub fn print_algo_info_box(algo: &dyn AlgorithmRunner) {
    let variants_str = algo.available_variants().join(", ");
    let name_line = format!("Algorithm: {}", algo.name());
    let cat_line = format!("Category:  {}", algo.category());
    let desc_line = algo.description();
    let var_line = format!("Variants: {}", variants_str);

    let content_width = [
        name_line.len(),
        cat_line.len(),
        desc_line.len(),
        var_line.len(),
    ]
    .iter()
    .cloned()
    .max()
    .unwrap_or(60);

    let border = "─".repeat(content_width + 2);

    println!("┌{}┐", border);
    println!("│ {:<width$} │", name_line, width = content_width);
    println!("│ {:<width$} │", cat_line, width = content_width);
    println!("│ {:<width$} │", desc_line, width = content_width);
    println!("├{}┤", border);
    println!("│ {:<width$} │", var_line, width = content_width);
    println!("└{}┘", border);
    println!();
}

/// Print results table for a single size
pub fn print_results_table(results: &[BenchmarkResult], size: usize, iterations: usize) {
    if results.is_empty() {
        return;
    }

    let baseline_time = results
        .first()
        .map(|r| r.avg_time.as_nanos() as f64)
        .unwrap_or(1.0);

    let baseline_result = results.first().map(|r| r.result_sample).unwrap_or(0.0);

    println!("  Size: {} ({} iterations)", size, iterations);
    println!("    {}", "─".repeat(100));
    println!(
        "    {:<30} {:>10} {:>10} {:>10} {:>10} {:>12} {:>12}",
        "Variant", "Average", "Min", "Max", "Speedup", "CV", "Rel. Error"
    );
    println!("    {}", "─".repeat(100));

    for result in results {
        let speedup = baseline_time / result.avg_time.as_nanos() as f64;

        let avg_ns = result.avg_time.as_nanos() as f64;
        let std_dev_ns = result.std_dev.as_nanos() as f64;

        let cv = if avg_ns > 0.0 {
            std_dev_ns / avg_ns
        } else {
            0.0
        };

        let diff = (result.result_sample - baseline_result).abs();
        let relative_error = if baseline_result.abs() > 1e-9 {
            diff / baseline_result.abs()
        } else {
            diff
        };

        let display_name = match &result.compiler {
            Some(c) => format!("{} ({})", result.variant_name, c),
            None => result.variant_name.clone(),
        };

        println!(
            "    {:<30} {:>10.2?} {:>10.2?} {:>10.2?} {:>9.2}x {:>11.2}% {:>12.2e}",
            display_name,
            result.avg_time,
            result.min_time,
            result.max_time,
            speedup,
            cv * 100.0,
            relative_error,
        );
    }
    println!();
}

/// Print the application header
pub fn print_header() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              Micro-Optimize-Algo Benchmarks                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
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
    println!("  --iter N       Number of iterations per benchmark (default: 10000)");
    println!("  --seed N       Random seed for reproducible benchmarks (default: time-based)");
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

/// Run multiple algorithms with randomized execution order and display results.
/// If csv_path is provided, also exports raw data to CSV.
pub fn run_all_algorithms_randomized(
    algorithms: &[&dyn AlgorithmRunner],
    sample_sizes: &[usize],
    iterations: usize,
    seed: Option<u64>,
    csv_path: Option<&str>,
) {
    let grouped = runner::run_all_algorithms_randomized(algorithms, sample_sizes, iterations, seed);

    // Export CSV if requested
    if let Some(path) = csv_path {
        match runner::export_csv(path, &grouped.raw_data) {
            Ok(()) => println!("  Raw data exported to: {}", path),
            Err(e) => eprintln!("  Warning: Failed to export CSV: {}", e),
        }
        println!();
    }

    // Display results grouped by algorithm and size
    for (algo_idx, algo) in algorithms.iter().enumerate() {
        print_algo_info_box(*algo);

        for (size_idx, &sample_size) in sample_sizes.iter().enumerate() {
            let mut variant_results = grouped.results[algo_idx][size_idx].clone();
            sort_variants(&mut variant_results);

            if !variant_results.is_empty() {
                print_results_table(&variant_results, sample_size, iterations);
            }
        }
    }
}

/// Run a single algorithm benchmark and display results
pub fn run_and_display(algo: &dyn AlgorithmRunner, sample_sizes: &[usize], iterations: usize) {
    print_algo_info_box(algo);

    for &sample_size in sample_sizes {
        let mut results = algo.run_benchmarks(sample_size, iterations);
        sort_variants(&mut results);
        print_results_table(&results, sample_size, iterations);
    }
}
