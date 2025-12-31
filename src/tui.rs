//! Text User Interface (TUI) utilities.
//!
//! Handles formatted output for the CLI.

use crate::registry::{AlgorithmRegistry, AlgorithmRunner};

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
    println!();
    println!("Arguments:");
    println!("  ALGORITHM      Name of specific algorithm to run (omit for all)");
    println!();
    println!("Examples:");
    println!("  micro-algo                    # Run all algorithms");
    println!("  micro-algo dot_product        # Run only dot_product");
    println!("  micro-algo --list             # List algorithms");
    println!("  micro-algo --sizes 128,512    # Custom sizes");
}

/// Print the list of available algorithms
pub fn print_available_algorithms(registry: &AlgorithmRegistry) {
    println!("Available algorithms:");
    println!();
    for algo in registry.all() {
        println!("  {:<20} [{}] - {}", 
            algo.name(), 
            algo.category(),
            algo.description()
        );
    }
}

/// Run an algorithm benchmark and display results formatted in a table
pub fn run_and_display(
    algo: &dyn AlgorithmRunner,
    sizes: &[usize],
    iterations: usize,
) {
    let variants_str = algo.available_variants().join(", ");
    let name_line = format!("Algorithm: {}", algo.name());
    let cat_line = format!("Category:  {}", algo.category());
    let desc_line = algo.description();
    let var_line = format!("Variants: {}", variants_str);

    let content_width = [name_line.len(), cat_line.len(), desc_line.len(), var_line.len()]
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

    for &size in sizes {
        let results = algo.run_benchmarks(size, iterations);
        if results.is_empty() {
             continue;
        }
        
        // Find baseline (first variant, usually "original")
        let baseline_time = results.first()
            .map(|r| r.avg_time.as_nanos() as f64)
            .unwrap_or(1.0);
        
        println!("  Algorithm: {}", algo.name());
        println!("  Size: {} ({} iterations)", size, iterations);
        println!("    {}", "─".repeat(100));
        println!(
            "    {:<30} {:>10} {:>10} {:>10} {:>10} {:>12} {:>12}",
            "Variant", "Average", "Min", "Max", "Speedup", "Time Var.", "Rel. Error"
        );
        println!("    {}", "─".repeat(100));
        
        let baseline_result = results.first()
            .map(|r| r.result_sample)
            .unwrap_or(0.0);

        for result in &results {
            let speedup = baseline_time / result.avg_time.as_nanos() as f64;
            
            // Calculate Time Variation: (Max - Min) / Avg
            let avg_ns = result.avg_time.as_nanos() as f64;
            let min_ns = result.min_time.as_nanos() as f64;
            let max_ns = result.max_time.as_nanos() as f64;
            
            let time_var = if avg_ns > 0.0 {
                (max_ns - min_ns) / avg_ns
            } else {
                0.0
            };

            // Calculate Relative Error
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
                time_var * 100.0,
                relative_error,
            );
        }
        println!();
    }
}
