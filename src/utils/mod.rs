//! Utility modules for benchmarking and execution.

pub mod bench;
pub mod runner;
pub mod timer;

#[cfg(feature = "cpu_cycles")]
pub mod cycles;

// Re-export commonly used items
pub use bench::{
    calculate_std_dev, compute_stats, shuffle, shuffle_with_rng, time_seed, SeededRng,
};
pub use runner::run_all_algorithms_randomized;
pub use timer::{
    calculate_median, calibrate, measure, measure_batched, TimingConfig, TimingResult,
};

#[cfg(not(feature = "cpu_cycles"))]
pub use bench::{elapsed, now};
#[cfg(feature = "cpu_cycles")]
pub use cycles::{measure_cycles, read_cycles};

/// C compiler name detected at build time
pub const C_COMPILER_NAME: Option<&str> = option_env!("C_COMPILER_NAME");

/// Information about an algorithm implementation variant.
/// Generic over F which is the function signature.
pub struct VariantInfo<F> {
    /// Unique identifier for this variant (e.g., "original", "x86_64-avx2")
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// The specific implementation function
    pub function: F,
}
