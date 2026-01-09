//! High-precision timing utilities with adaptive batching.
//!
//! This module provides timing infrastructure for micro-benchmarks with:
//! - Adaptive batching to handle very fast operations
//! - All raw measurements preserved for external analysis
//! - Support for large iteration counts

use std::hint::black_box;
use std::time::{Duration, Instant};

/// Configuration for timing measurements
#[derive(Clone, Debug)]
pub struct TimingConfig {
    /// Target minimum duration per sample for stable measurements (default: 1ms)
    pub min_sample_duration: Duration,
    /// Minimum number of samples to collect (default: 30)
    pub min_samples: usize,
    /// Number of warmup iterations before calibration (default: 10)
    pub warmup_iterations: usize,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            min_sample_duration: Duration::from_millis(1),
            min_samples: 30,
            warmup_iterations: 10,
        }
    }
}

/// Result from timing measurements - all raw data preserved
#[derive(Clone, Debug)]
pub struct TimingResult {
    /// Arithmetic mean of all samples
    pub mean: Duration,
    /// Median of all samples
    pub median: Duration,
    /// Minimum sample time
    pub min: Duration,
    /// Maximum sample time
    pub max: Duration,
    /// Standard deviation
    pub std_dev: Duration,
    /// Number of samples collected
    pub sample_count: usize,
    /// Iterations per sample (batch size used)
    pub iterations_per_sample: usize,
    /// All raw sample durations (per-iteration, not per-batch)
    pub raw_samples: Vec<Duration>,
}

/// Calibrate the batch size for a closure to achieve stable measurements.
///
/// This determines how many iterations should be batched together to get
/// measurements in the target duration range (typically ~1ms).
///
/// # Arguments
/// * `func` - The function to calibrate. Must be callable and return a value.
/// * `config` - Timing configuration with target duration.
///
/// # Returns
/// The optimal batch size (iterations per timing call).
pub fn calibrate<F, R>(mut func: F, config: &TimingConfig) -> usize
where
    F: FnMut() -> R,
{
    // Warmup
    for _ in 0..config.warmup_iterations {
        black_box(func());
    }

    let mut batch_size = 1usize;
    let max_batch_size = 1_000_000_000; // Prevent infinite loops

    loop {
        let start = Instant::now();
        for _ in 0..batch_size {
            black_box(func());
        }
        let elapsed = start.elapsed();

        if elapsed >= config.min_sample_duration || batch_size >= max_batch_size {
            return batch_size;
        }

        // Double batch size, but don't overflow
        batch_size = batch_size.saturating_mul(2).min(max_batch_size);
    }
}

/// Measure a closure with adaptive batching.
///
/// This function:
/// 1. Calibrates the batch size to get stable measurements
/// 2. Collects the requested number of samples
/// 3. Returns all raw data for external analysis
///
/// # Arguments
/// * `func` - The function to measure.
/// * `total_iterations` - Total number of iterations to perform.
/// * `config` - Timing configuration.
///
/// # Returns
/// A `TimingResult` with all statistics and raw samples preserved.
pub fn measure<F, R>(mut func: F, total_iterations: usize, config: &TimingConfig) -> TimingResult
where
    F: FnMut() -> R,
{
    // Calibrate batch size
    let batch_size = calibrate(&mut func, config);

    // Calculate number of samples
    let num_samples = (total_iterations / batch_size).max(config.min_samples);

    // Collect samples
    let mut raw_batch_times: Vec<Duration> = Vec::with_capacity(num_samples);

    for _ in 0..num_samples {
        let start = Instant::now();
        for _ in 0..batch_size {
            black_box(func());
        }
        raw_batch_times.push(start.elapsed());
    }

    // Convert batch times to per-iteration times
    let raw_samples: Vec<Duration> = raw_batch_times
        .iter()
        .map(|&batch_time| batch_time / batch_size as u32)
        .collect();

    // Compute statistics
    compute_timing_result(raw_samples, batch_size)
}

/// Measure a closure that already handles its own iteration loop.
///
/// Use this when the closure runs `batch_size` iterations internally.
///
/// # Arguments
/// * `func` - Function that takes batch_size and returns (result, elapsed).
/// * `batch_size` - Number of iterations per call to func.
/// * `num_samples` - Number of samples to collect.
///
/// # Returns
/// A `TimingResult` with all statistics and raw samples preserved.
pub fn measure_batched<F, R>(mut func: F, batch_size: usize, num_samples: usize) -> TimingResult
where
    F: FnMut(usize) -> (R, Duration),
{
    let mut raw_batch_times: Vec<Duration> = Vec::with_capacity(num_samples);

    for _ in 0..num_samples {
        let (result, elapsed) = func(batch_size);
        black_box(result);
        raw_batch_times.push(elapsed);
    }

    // Convert batch times to per-iteration times
    let raw_samples: Vec<Duration> = raw_batch_times
        .iter()
        .map(|&batch_time| batch_time / batch_size as u32)
        .collect();

    compute_timing_result(raw_samples, batch_size)
}

/// Compute timing result from raw samples (all data preserved, no filtering).
fn compute_timing_result(raw_samples: Vec<Duration>, iterations_per_sample: usize) -> TimingResult {
    if raw_samples.is_empty() {
        return TimingResult {
            mean: Duration::ZERO,
            median: Duration::ZERO,
            min: Duration::ZERO,
            max: Duration::ZERO,
            std_dev: Duration::ZERO,
            sample_count: 0,
            iterations_per_sample,
            raw_samples: Vec::new(),
        };
    }

    // Sort for median calculation
    let mut sorted = raw_samples.clone();
    sorted.sort();

    let min = sorted[0];
    let max = sorted[sorted.len() - 1];
    let median = sorted[sorted.len() / 2];

    // Mean
    let total: Duration = raw_samples.iter().sum();
    let mean = total / raw_samples.len() as u32;

    // Standard deviation
    let std_dev = calculate_std_dev(&raw_samples, mean);

    TimingResult {
        mean,
        median,
        min,
        max,
        std_dev,
        sample_count: raw_samples.len(),
        iterations_per_sample,
        raw_samples,
    }
}

/// Calculate standard deviation from a list of durations.
fn calculate_std_dev(times: &[Duration], mean: Duration) -> Duration {
    if times.len() < 2 {
        return Duration::ZERO;
    }

    let mean_ns = mean.as_nanos() as f64;
    let variance: f64 = times
        .iter()
        .map(|t| {
            let diff = t.as_nanos() as f64 - mean_ns;
            diff * diff
        })
        .sum::<f64>()
        / (times.len() - 1) as f64;

    let std_dev_ns = variance.sqrt();
    Duration::from_nanos(std_dev_ns as u64)
}

/// Calculate median from a slice of durations.
pub fn calculate_median(times: &[Duration]) -> Duration {
    if times.is_empty() {
        return Duration::ZERO;
    }
    let mut sorted: Vec<_> = times.to_vec();
    sorted.sort();
    sorted[sorted.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibrate_fast_function() {
        let config = TimingConfig::default();
        let batch_size = calibrate(|| 1 + 1, &config);
        // A trivial operation should need many batched iterations
        assert!(
            batch_size > 1000,
            "Expected large batch size for fast op, got {}",
            batch_size
        );
    }

    #[test]
    fn test_measure_preserves_all_samples() {
        let config = TimingConfig {
            min_sample_duration: Duration::from_micros(100),
            min_samples: 10,
            warmup_iterations: 5,
        };

        let result = measure(|| std::hint::black_box(42), 1000, &config);

        assert!(result.sample_count >= 10);
        assert_eq!(result.raw_samples.len(), result.sample_count);
    }

    #[test]
    fn test_timing_result_statistics() {
        let samples = vec![
            Duration::from_nanos(100),
            Duration::from_nanos(200),
            Duration::from_nanos(150),
        ];

        let result = compute_timing_result(samples, 1);

        assert_eq!(result.min, Duration::from_nanos(100));
        assert_eq!(result.max, Duration::from_nanos(200));
        assert_eq!(result.median, Duration::from_nanos(150));
        assert_eq!(result.sample_count, 3);
    }
}
