//! Algorithm registry for dynamic algorithm discovery and execution.
//!
//! This module provides a generic interface for registering and running
//! algorithms without needing separate binary files for each.

use crate::utils::bench::Measurement;
use crate::utils::timer::VariantResult;

/// Result from running a variant benchmark (alias for VariantResult)
pub type BenchmarkResult = VariantResult;

/// A simple closure that runs one iteration of a variant
pub struct VariantClosure<'a> {
    pub name: &'static str,
    pub description: &'static str,
    /// Returns (timing_measurement, optional_result_value).
    /// Timing happens inside the closure to eliminate Fn trait overhead.
    pub run: Box<dyn FnMut() -> (Measurement, Option<f64>) + 'a>,
}

/// Trait that all algorithm benchmarkers must implement
pub trait AlgorithmRunner: Send + Sync {
    /// Name of the algorithm (e.g., "dot_product")
    fn name(&self) -> &'static str;

    /// Human-readable description
    fn description(&self) -> &'static str;

    /// Category (e.g., "math", "sorting")
    fn category(&self) -> &'static str;

    /// Get list of available variant names
    fn available_variants(&self) -> Vec<&'static str>;

    /// Get closures for each variant, ready to be measured.
    /// Each closure does ONE execution and returns a result value.
    /// The runner will handle warmup, timing, and repetition.
    fn get_variant_closures<'a>(&'a self, size: usize) -> Vec<VariantClosure<'a>>;

    /// Verify correctness of all variants against the reference
    fn verify(&self) -> Result<(), String>;
}

/// Global registry of all algorithms
pub struct AlgorithmRegistry {
    algorithms: Vec<Box<dyn AlgorithmRunner>>,
}

impl AlgorithmRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            algorithms: Vec::new(),
        }
    }

    /// Register an algorithm
    pub fn register<A: AlgorithmRunner + 'static>(&mut self, algo: A) {
        self.algorithms.push(Box::new(algo));
    }

    /// Get all registered algorithms
    pub fn all(&self) -> &[Box<dyn AlgorithmRunner>] {
        &self.algorithms
    }

    /// Find algorithm by name
    pub fn find(&self, name: &str) -> Option<&dyn AlgorithmRunner> {
        self.algorithms
            .iter()
            .find(|a| a.name() == name)
            .map(|a| a.as_ref())
    }

    /// List algorithm names
    pub fn list_names(&self) -> Vec<&'static str> {
        self.algorithms.iter().map(|a| a.name()).collect()
    }

    /// List algorithms by category
    pub fn by_category(&self, category: &str) -> Vec<&dyn AlgorithmRunner> {
        self.algorithms
            .iter()
            .filter(|a| a.category() == category)
            .map(|a| a.as_ref())
            .collect()
    }
}

impl Default for AlgorithmRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the default registry with all algorithms
pub fn build_registry() -> AlgorithmRegistry {
    let mut registry = AlgorithmRegistry::new();

    // Register all algorithms here
    registry.register(crate::math::dot_product::DotProductRunner);
    registry.register(crate::random::xoroshiro::XoroshiroRunner);
    registry.register(crate::control_flow::call_vs_branch::CallVsBranchRunner);
    registry.register(crate::control_flow::elseif_vs_jumptable::ElseIfVsJumpTableRunner);

    registry
}
