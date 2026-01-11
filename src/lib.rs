//! # Micro-Optimize-Algo
//!
//! A collection of algorithms that have been micro-optimized using various techniques.

pub mod control_flow;
pub mod math;
pub mod random;
pub mod registry;
pub mod utils;

/// Re-export tui from utils for backward compatibility
pub use utils::tui;

/// Re-export run_benchmarks from utils::runner
pub use utils::runner::run_benchmarks;

/// Re-export commonly used items
pub mod prelude {
    pub use crate::math::dot_product;
    pub use crate::registry::{build_registry, AlgorithmRegistry, AlgorithmRunner};
}

#[cfg(test)]
mod tests {
    use crate::registry::build_registry;

    #[test]
    fn test_all_algorithms_registry_verify() {
        let registry = build_registry();
        let algorithms = registry.all();

        println!("Verifying {} algorithms...", algorithms.len());

        for algo in algorithms {
            println!("Verifying algorithm: {}", algo.name());
            match algo.verify() {
                Ok(_) => println!("  ✅ Algorithm '{}' passed verification", algo.name()),
                Err(e) => panic!(
                    "  ❌ Algorithm '{}' failed verification: {}",
                    algo.name(),
                    e
                ),
            }
        }
    }
}
