//! Text User Interface (TUI) utilities for micro-benchmarks.

use terminal_size::{terminal_size, Width};

/// Get the current terminal width, constrained to a reasonable range
fn get_term_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        (w as usize).clamp(40, 200)
    } else {
        80
    }
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
    println!("  --list, -l       List all available algorithms");
    println!("  --help, -h       Show this help message");
    println!("  --sizes SIZES    Comma-separated input sizes (default: 64,256,1024,4096,16384)");
    println!("  --iter, -r N     Number of runs per variant (default: 30)");
    println!("  --seed N         Random seed for reproducible benchmarks (default: time-based)");
    println!("  --csv FILE       Export timing data to CSV file");
    println!("  --filter, -f     Filter outliers (trim 1% extremes from measurements)");
    println!("  --pin MODE       CPU pin strategy: 'global' or 'per-call' (default: per-call)");
    println!();
    println!("Arguments:");
    println!("  ALGORITHM        Name of specific algorithm to run (omit for all)");
    println!();
    println!("Examples:");
    println!("  micro-algo                      # Run all algorithms");
    println!("  micro-algo dot_product          # Run only dot_product");
    println!("  micro-algo --list               # List algorithms");
    println!("  micro-algo --sizes 128,512      # Custom sizes");
    println!("  micro-algo --seed 12345         # Reproducible run");
    println!("  micro-algo --csv data.csv       # Export timings to CSV");
    println!("  micro-algo --pin global         # Use global CPU pinning");
}

/// Print the list of available algorithms
pub fn print_available_algorithms() {
    println!("Available algorithms:");
    println!();
    println!(
        "  {:<20} - Computes the sum of products of corresponding vector elements",
        "dot_product"
    );
    println!(
        "  {:<20} - Computes Perlin gradient dot products over slices of positions",
        "grad_dot"
    );
    println!(
        "  {:<20} - Xoroshiro128++ pseudo-random number generator",
        "xoroshiro128++"
    );
    println!(
        "  {:<20} - Comparison between function calls (CALL/RET) and inline code",
        "call_vs_branch"
    );
    println!(
        "  {:<20} - Comparison between else-if chains and jump tables",
        "elseif_vs_jumptable"
    );
    println!(
        "  {:<20} - Compare iteration methods over object pools with manual vtables",
        "vtable_iteration"
    );
}
