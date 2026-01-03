//! Algorithm registry for dynamic algorithm discovery and execution.
//!
//! This module provides a generic interface for registering and running
//! algorithms without needing separate binary files for each.

use std::time::Duration;

/// Result from running a variant benchmark
#[derive(Clone)]
pub struct BenchmarkResult {
    pub variant_name: String,
    pub description: String,
    pub avg_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub std_dev: Duration,  // Standard deviation of timing measurements
    pub iterations: usize,

    pub result_sample: f64,
    pub compiler: Option<String>,
}

/// A benchmark closure - a function that runs one iteration and returns result + timing
pub struct BenchmarkClosure {
    pub name: &'static str,
    pub description: &'static str,
    pub compiler: Option<&'static str>,
    /// The actual benchmark function - runs one iteration, returns (result, elapsed_time)
    /// Each implementation measures its own time internally to exclude FFI overhead for C variants
    pub run: Box<dyn FnMut() -> (f64, Duration) + Send>,
}

/// Trait that all algorithm benchmarkers must implement
pub trait AlgorithmRunner: Send + Sync {
    /// Name of the algorithm (e.g., "dot_product")
    fn name(&self) -> &'static str;
    
    /// Human-readable description
    fn description(&self) -> &'static str;
    
    /// Category (e.g., "math", "sorting")
    fn category(&self) -> &'static str;
    
    /// Run benchmarks for all variants at a given input size (legacy method)
    fn run_benchmarks(&self, size: usize, iterations: usize) -> Vec<BenchmarkResult>;
    
    /// Get list of available variant names
    fn available_variants(&self) -> Vec<&'static str>;

    /// Verify correctness of all variants against the reference
    fn verify(&self) -> Result<(), String>;
    
    /// Get benchmark closures for randomized execution
    /// Each closure runs one iteration of one variant
    /// The seed is used to generate reproducible test data
    fn get_benchmark_closures(&self, size: usize, seed: u64) -> Vec<BenchmarkClosure>;
    
    /// Warmup all variants
    /// The seed is used to generate reproducible test data
    fn warmup(&self, size: usize, warmup_iterations: usize, seed: u64);
}

/// Global registry of all algorithms
pub struct AlgorithmRegistry {
    algorithms: Vec<Box<dyn AlgorithmRunner>>,
}

impl AlgorithmRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self { algorithms: Vec::new() }
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
        self.algorithms.iter()
            .find(|a| a.name() == name)
            .map(|a| a.as_ref())
    }
    
    /// List algorithm names
    pub fn list_names(&self) -> Vec<&'static str> {
        self.algorithms.iter().map(|a| a.name()).collect()
    }
    
    /// List algorithms by category
    pub fn by_category(&self, category: &str) -> Vec<&dyn AlgorithmRunner> {
        self.algorithms.iter()
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

