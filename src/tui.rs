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
    let name = result.variant_name.to_lowercase();
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
    results.sort_by(|a, b| variant_sort_key(a).cmp(&variant_sort_key(b)));
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
pub fn print_results_table(results: &[BenchmarkResult], size: usize, iterations: usize) {
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

    let baseline_result = results.first().map(|r| r.result_sample).unwrap_or(0.0);

    println!("  Size: {} ({} iterations)", size, iterations);
    println!("  {}", "─".repeat(table_width));
    println!(
        "  {:<v_width$} {:>12} {:>12} {:>12} {:>9} {:>9} {:>10}",
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

        let diff = (result.result_sample - baseline_result).abs();
        let relative_error = if baseline_result.abs() > 1e-9 {
            diff / baseline_result.abs()
        } else {
            diff
        };

        let display_name =
            if result.variant_name.starts_with("c-") || result.variant_name.starts_with("c_") {
                match crate::utils::C_COMPILER_NAME {
                    Some(c) => format!("{} ({})", result.variant_name, c),
                    None => result.variant_name.clone(),
                }
            } else {
                result.variant_name.clone()
            };

        #[cfg(all(feature = "cpu_cycles", not(feature = "use_time")))]
        let time_str = format!("{} {}", avg_ns as u64, crate::utils::bench::unit_name());

        #[cfg(any(not(feature = "cpu_cycles"), feature = "use_time"))]
        let time_str = format!("{:?}", result.avg_time);

        #[cfg(all(feature = "cpu_cycles", not(feature = "use_time")))]
        let (min_str, max_str) = (
            format!("{}", result.min_time.as_nanos() as u64),
            format!("{}", result.max_time.as_nanos() as u64),
        );

        #[cfg(any(not(feature = "cpu_cycles"), feature = "use_time"))]
        let (min_str, max_str) = (
            format!("{:?}", result.min_time),
            format!("{:?}", result.max_time),
        );

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
