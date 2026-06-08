//! # Micro-Optimize-Algo
//!
//! A collection of algorithms that have been micro-optimized using various techniques.

#![feature(portable_simd)]

pub mod control_flow;
pub mod math;
pub mod random;
pub mod registry;
pub mod utils;

// Re-export the proc macros
pub use micro_optimize_algo_macros::{scenario, variant};

/// Re-export run_benchmarks from utils::runner
pub use utils::runner::run_benchmarks;

/// Re-export Scenario traits and types
pub use utils::scenario::{CpuTimer, Scenario, ScenarioDefinition};

#[cfg(test)]
mod tests {
    use crate::registry::{get_run_metadata, init_session, run_scenario};
    use crate::utils::runner::Config;

    #[test]
    fn test_all_algorithms_registry_verify() {
        let config = Config {
            sample: Some(1),
            ..Default::default()
        };
        let session = init_session(&config);
        println!("Verifying {} runs...", session.run_ids.len());
        for &run_id in &session.run_ids {
            let (_time, other) = run_scenario(&session, run_id);
            let meta = get_run_metadata(&session, run_id);
            println!(
                "  Verified: {}::{} (variant: {}, context: {}) -> result: {:?}",
                meta.group, meta.scenario, meta.variant, meta.context, other
            );
        }
    }
}
