//! Generic CLI for running algorithms.
//!
//! Usage:
//!   micro-algo              # Run all algorithms
//!   micro-algo --list       # List available algorithms
//!   micro-algo dot_product  # Run specific algorithm
//!   micro-algo --help       # Show help

use micro_optimize_algo::registry::build_registry;
use std::env;



fn main() {
    let args: Vec<String> = env::args().collect();
    let registry = build_registry();
    
    // Parse arguments
    let mut show_list = false;
    let mut show_help = false;
    let mut sizes: Vec<usize> = vec![64, 256, 1024, 4096, 16384];
    let mut iterations: usize = 10000;
    let mut algorithm_filter: Option<String> = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--list" | "-l" => show_list = true,
            "--help" | "-h" => show_help = true,
            "--sizes" => {
                i += 1;
                if i < args.len() {
                    sizes = args[i]
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();
                }
            }
            "--iter" => {
                i += 1;
                if i < args.len() {
                    iterations = args[i].parse().unwrap_or(10000);
                }
            }
            arg if !arg.starts_with('-') => {
                algorithm_filter = Some(arg.to_string());
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }
    
    if show_help {
        micro_optimize_algo::tui::print_help();
        return;
    }
    
    if show_list {
        micro_optimize_algo::tui::print_available_algorithms(&registry);
        return;
    }
    
    micro_optimize_algo::tui::print_header();
    
    match algorithm_filter {
        Some(name) => {
            match registry.find(&name) {
                Some(algo) => micro_optimize_algo::tui::run_and_display(algo, &sizes, iterations),
                None => {
                    eprintln!("Algorithm '{}' not found.", name);
                    eprintln!("Available: {:?}", registry.list_names());
                    std::process::exit(1);
                }
            }
        }
        None => {
            for algo in registry.all() {
                micro_optimize_algo::tui::run_and_display(algo.as_ref(), &sizes, iterations);
            }
        }
    }
    
    println!("Note: Speedup is relative to the first variant (usually 'original').");
}
