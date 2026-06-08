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

    let mut config_path: Option<String> = None;
    let mut runs_override: Option<u64> = None;
    let mut csv_path: Option<String> = None;
    let mut filter_outliers = false;
    let mut filter_scenario: Option<Vec<String>> = None;
    let mut filter_variant: Option<Vec<String>> = None;
    let mut filter_tag: Option<Vec<String>> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
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
                    if let Ok(n) = args[i].parse::<u64>() {
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
                println!("  -c, --config <file>   Load config from TOML file (default: config.toml if exists)");
                println!("  -r, --runs <number>   Override number of sample runs");
                println!("  --csv <file>          Path to save CSV output");
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

    // Apply CLI overrides
    if let Some(r) = runs_override {
        config.sample = Some(r);
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

    micro_optimize_algo::utils::runner::run_benchmarks(
        config,
        csv_path.as_deref(),
        filter_outliers,
    );
}
