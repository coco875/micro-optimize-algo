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

#[cfg(feature = "cpu_cycles")]
pub use cycles::{measure_cycles, read_cycles};
