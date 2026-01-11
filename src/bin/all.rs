//! Generic CLI for running algorithms.
//!
//! Usage:
//!   micro-algo              # Run all algorithms
//!   micro-algo --list       # List available algorithms
//!   micro-algo dot_product  # Run specific algorithm
//!   micro-algo --help       # Show help

use micro_optimize_algo::registry::build_registry;
use micro_optimize_algo::PinStrategy;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let registry = build_registry();

    // Parse arguments
    let mut show_list = false;
    let mut show_help = false;
    let mut sample_sizes: Vec<usize> = vec![64, 256, 1024, 4096, 16384];
    let mut runs: usize = 30;
    let mut seed: Option<u64> = None;
    let mut csv_path: Option<String> = None;
    let mut algorithm_filter: Option<String> = None;
    let mut filter_outliers: bool = false;
    let mut pin_strategy: PinStrategy = PinStrategy::PerExecution;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--list" | "-l" => show_list = true,
            "--help" | "-h" => show_help = true,
            "--sizes" => {
                i += 1;
                if i < args.len() {
                    sample_sizes = args[i]
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();
                }
            }
            "--iter" | "--runs" | "-r" => {
                i += 1;
                if i < args.len() {
                    runs = args[i].parse().unwrap_or(30);
                }
            }
            "--seed" => {
                i += 1;
                if i < args.len() {
                    seed = args[i].parse().ok();
                }
            }
            "--csv" => {
                i += 1;
                if i < args.len() {
                    csv_path = Some(args[i].clone());
                }
            }
            "--filter" | "-f" => {
                filter_outliers = true;
            }
            "--pin" => {
                i += 1;
                if i < args.len() {
                    pin_strategy = match args[i].as_str() {
                        "global" => PinStrategy::Global,
                        "per-call" | "per-execution" => PinStrategy::PerExecution,
                        other => {
                            eprintln!("Unknown pin strategy '{}'. Use 'global' or 'per-call'.", other);
                            std::process::exit(1);
                        }
                    };
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
            // Running a single algorithm
            match registry.find(&name) {
                Some(algo) => {
                    let algos = vec![algo];
                    micro_optimize_algo::run_benchmarks(
                        &algos,
                        &sample_sizes,
                        runs,
                        seed,
                        csv_path.as_deref(),
                        filter_outliers,
                        pin_strategy,
                    );
                }
                None => {
                    eprintln!("Algorithm '{}' not found.", name);
                    eprintln!("Available: {:?}", registry.list_names());
                    std::process::exit(1);
                }
            }
        }
        None => {
            // Running all algorithms
            let all_algos: Vec<_> = registry.all().iter().map(|a| a.as_ref()).collect();
            micro_optimize_algo::run_benchmarks(
                &all_algos,
                &sample_sizes,
                runs,
                seed,
                csv_path.as_deref(),
                filter_outliers,
                pin_strategy,
            );
        }
    }

    println!("Note: Speedup is relative to the first variant (usually 'original').");
}
