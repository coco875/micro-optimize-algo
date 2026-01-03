//! # Micro-Optimize-Algo
//!
//! A collection of algorithms that have been micro-optimized using various techniques.

pub mod math;
pub mod registry;
pub mod tui;
pub mod random;
pub mod control_flow;
pub mod utils;

/// Re-export commonly used items
pub mod prelude {
    pub use crate::math::dot_product;
    pub use crate::registry::{AlgorithmRegistry, AlgorithmRunner, build_registry};
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
                Err(e) => panic!("  ❌ Algorithm '{}' failed verification: {}", algo.name(), e),
            }
        }
    }
}
