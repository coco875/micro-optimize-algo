//! Utility modules for benchmarking and execution.

pub mod bench;
pub mod runner;
pub mod timer;

// Re-export commonly used items
pub use bench::{calculate_std_dev, compute_stats, shuffle, shuffle_with_rng, time_seed, SeededRng};
pub use runner::run_all_algorithms_randomized;
pub use timer::{TimingConfig, TimingResult, calibrate, measure, measure_batched, calculate_median};

