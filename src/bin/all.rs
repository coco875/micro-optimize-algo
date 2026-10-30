//! Generic CLI for running algorithms.
//!
//! Usage:
//!   micro-algo                    # Run according to config/defaults
//!   micro-algo --config config.toml # Run with specific config
//!   micro-algo --runs 30           # Override runs count
//!   micro-algo --csv results.csv   # Save output to CSV

use micro_optimize_algo::utils::runner::Config;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut verify = false;
    let mut seed_override = None;
    let mut config_path: Option<String> = None;
    let mut runs_override: Option<u64> = None;
    let mut warmup_override = None;
    let mut pin_override = None;
    let mut csv_path: Option<String> = None;
    let mut filter_outliers = false;
    let mut filter_scenario: Option<Vec<String>> = None;
    let mut filter_variant: Option<Vec<String>> = None;
    let mut filter_tag: Option<Vec<String>> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--verify" => {
                verify = true;
            }
            "--seed" => {
                i += 1;
                seed_override = Some(
                    args.get(i)
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or_else(|| {
                            eprintln!("Error: --seed requires an unsigned integer");
                            std::process::exit(1);
                        }),
                );
            }
            "--config" | "-c" => {
                i += 1;
                if i < args.len() {
                    config_path = Some(args[i].clone());
                } else {
                    eprintln!("Error: --config requires a path");
                    std::process::exit(1);
                }
            }
            "--runs" | "--iter" | "-r" => {
                i += 1;
                if i < args.len() {
                    if let Some(n) = args[i].parse::<u64>().ok().filter(|n| *n > 0) {
                        runs_override = Some(n);
                    } else {
                        eprintln!("Error: --runs requires a positive integer");
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("Error: --runs requires a value");
                    std::process::exit(1);
                }
            }
            "--warmup" => {
                i += 1;
                warmup_override = Some(
                    args.get(i)
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or_else(|| {
                            eprintln!("Error: --warmup requires a non-negative integer");
                            std::process::exit(1);
                        }),
                );
            }
            "--pin" => {
                i += 1;
                pin_override =
                    Some(args.get(i).and_then(|s| s.parse().ok()).unwrap_or_else(|| {
                        eprintln!("Error: --pin requires 'global' or 'per-call'");
                        std::process::exit(1);
                    }));
            }
            "--csv" => {
                i += 1;
                if i < args.len() {
                    csv_path = Some(args[i].clone());
                } else {
                    eprintln!("Error: --csv requires a path");
                    std::process::exit(1);
                }
            }
            "--filter" | "-f" => {
                filter_outliers = true;
            }
            "--scenario" => {
                i += 1;
                if i < args.len() {
                    filter_scenario =
                        Some(args[i].split(',').map(|s| s.trim().to_string()).collect());
                } else {
                    eprintln!("Error: --scenario requires a comma-separated list");
                    std::process::exit(1);
                }
            }
            "--variant" => {
                i += 1;
                if i < args.len() {
                    filter_variant =
                        Some(args[i].split(',').map(|s| s.trim().to_string()).collect());
                } else {
                    eprintln!("Error: --variant requires a comma-separated list");
                    std::process::exit(1);
                }
            }
            "--tag" | "-t" => {
                i += 1;
                if i < args.len() {
                    filter_tag = Some(args[i].split(',').map(|s| s.trim().to_string()).collect());
                } else {
                    eprintln!("Error: --tag requires a comma-separated list");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                println!("Usage: micro-algo [options]");
                println!("Options:");
                println!(
                    "  --verify              Validate dedicated fixtures without benchmarking"
                );
                println!("  --seed <number>       Replay inputs and order (validation default: 0)");
                println!("  -c, --config <file>   Load config from TOML file (default: config.toml if exists)");
                println!("  -r, --runs <number>   Override number of sample runs (default: 30)");
                println!("  --warmup <number>     Warmup runs per variant/parameter (default: 10; 0 disables)");
                println!(
                    "  --pin <strategy>      CPU affinity: global or per-call (default: per-call)"
                );
                println!(
                    "  --csv <file>          Export timing statistics + .samples.csv, or validation checks with --verify"
                );
                println!("  -f, --filter          Filter timing outliers");
                println!("  --scenario <list>     Comma-separated list of scenarios to run");
                println!("  --variant <list>      Comma-separated list of variants to run");
                println!("  -t, --tag <list>      Comma-separated list of tags to run");
                println!("  -h, --help            Show this help menu");
                return;
            }
            arg if !arg.starts_with('-') => {
                // If it is a positional argument, treat it as a scenario filter for backward compatibility
                let list = filter_scenario.get_or_insert_with(Vec::new);
                list.push(arg.to_string());
            }
            _ => {
                eprintln!("Error: Unknown option '{}'", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // Try loading config.toml by default if no path is given
    let actual_config_path = config_path.or_else(|| {
        if std::path::Path::new("config.toml").exists() {
            Some("config.toml".to_string())
        } else {
            None
        }
    });

    let mut config = if let Some(path) = actual_config_path {
        println!("Loading config from: {}", path);
        let content = fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Error: Could not read config file '{}': {}", path, e);
            std::process::exit(1);
        });
        toml::from_str::<Config>(&content).unwrap_or_else(|e| {
            eprintln!("Error: Failed to parse TOML config from '{}': {}", path, e);
            std::process::exit(1);
        })
    } else {
        Config::default()
    };

    if let Some(seed) = seed_override {
        config.seed = Some(seed);
    }

    // Apply CLI overrides
    if let Some(r) = runs_override {
        config.sample = Some(r);
    }
    if let Some(warmup) = warmup_override {
        config.warmup = Some(warmup);
    }
    if let Some(pin) = pin_override {
        config.pin = pin;
    }
    if let Some(fs) = filter_scenario {
        config.filter_scenario = Some(fs);
    }
    if let Some(fv) = filter_variant {
        config.filter_variant = Some(fv);
    }
    if let Some(ft) = filter_tag {
        config.filter_tag = Some(ft);
    }

    if verify {
        println!("Validation seed: {}", config.seed.unwrap_or(0));
        let report = micro_optimize_algo::registry::verify(&config);
        report.print();
        if let Some(path) = csv_path {
            if let Err(error) = report.export_csv(&path) {
                eprintln!(
                    "Error: Failed to export validation CSV '{}': {}",
                    path, error
                );
                std::process::exit(1);
            }
            println!("Validation checks exported to: {}", path);
        }
        if !report.passed() {
            std::process::exit(1);
        }
        return;
    }

    micro_optimize_algo::utils::runner::run_benchmarks(
        config,
        csv_path.as_deref(),
        filter_outliers,
    );
}
