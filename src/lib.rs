//! # Micro-Optimize-Algo
//!
//! A collection of algorithms that have been micro-optimized using various techniques.

#![feature(portable_simd)]

include!(concat!(env!("OUT_DIR"), "/algorithms.rs"));
pub mod registry;
pub mod utils;

// Re-export the proc macros
pub use micro_optimize_algo_macros::{
    algorithm, algorithm_modules, implementation_modules, scenario, variant,
};

/// Re-export run_benchmarks from utils::runner
pub use utils::runner::run_benchmarks;

/// Re-export Scenario traits and types
pub use utils::scenario::{CpuTimer, Scenario, ScenarioDefinition};
