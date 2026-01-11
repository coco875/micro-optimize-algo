//! Text User Interface (TUI) utilities.
//!
//! Handles formatted output for the CLI.

use crate::registry::{AlgorithmRegistry, AlgorithmRunner, BenchmarkResult};
use terminal_size::{terminal_size, Width};

/// Get the current terminal width, constrained to a reasonable range
fn get_term_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        (w as usize).clamp(40, 200)
    } else {
        80
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
        (3, name.clone(), compiler)
    } else if name.starts_with("c-") || name.starts_with("c_") {
        (2, compiler.clone(), name.clone())
    } else {
        (1, name.clone(), String::new())
    }
}

/// Sort variants: original first, then grouped by language (Rust, C, ASM)
pub fn sort_variants(results: &mut [BenchmarkResult]) {
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

    let content_width = [name_line.len(), cat_line.len(), desc_line.len(), var_line.len()]
        .iter().cloned().max().unwrap_or(60).min(max_content_width);

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
    let fixed_width = 72;
    let variant_col_width = term_width.saturating_sub(fixed_width).max(15);
    let table_width = variant_col_width + 64 + 6;

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

        let relative_error = match (result.result_sample, baseline_result) {
            (Some(res), Some(base)) => {
                let diff = (res - base).abs();
                if base.abs() > 1e-9 { diff / base.abs() } else { diff }
            }
            _ => 0.0,
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
