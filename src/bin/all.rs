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
                    if sample_sizes.is_empty() {
                        eprintln!("Error: --sizes requires valid comma-separated integers");
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("Error: --sizes requires a value (e.g., --sizes 64,256,1024)");
                    std::process::exit(1);
                }
            }
            "--iter" | "--runs" | "-r" => {
                i += 1;
                if i < args.len() {
                    runs = match args[i].parse() {
                        Ok(n) if n > 0 => n,
                        _ => {
                            eprintln!("Error: --iter requires a positive integer");
                            std::process::exit(1);
                        }
                    };
                } else {
                    eprintln!("Error: --iter requires a value (e.g., --iter 1000)");
                    std::process::exit(1);
                }
            }
            "--seed" => {
                i += 1;
                if i < args.len() {
                    seed = match args[i].parse() {
                        Ok(n) => Some(n),
                        Err(_) => {
                            eprintln!("Error: --seed requires a valid integer");
                            std::process::exit(1);
                        }
                    };
                } else {
                    eprintln!("Error: --seed requires a value (e.g., --seed 12345)");
                    std::process::exit(1);
                }
            }
            "--csv" => {
                i += 1;
                if i < args.len() {
                    csv_path = Some(args[i].clone());
                } else {
                    eprintln!("Error: --csv requires a file path (e.g., --csv results.csv)");
                    std::process::exit(1);
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
                            eprintln!("Error: Unknown pin strategy '{}'. Use 'global' or 'per-call'.", other);
                            std::process::exit(1);
                        }
                    };
                } else {
                    eprintln!("Error: --pin requires a value (e.g., --pin global or --pin per-call)");
                    std::process::exit(1);
                }
            }
            arg if !arg.starts_with('-') => {
                algorithm_filter = Some(arg.to_string());
            }
            _ => {
                eprintln!("Error: Unknown option '{}'", args[i]);
                eprintln!("Use --help for usage information.");
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
