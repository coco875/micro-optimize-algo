//! Unified timing system for micro-benchmarks.
//!
//! This module provides the single timing infrastructure with:
//! - Support for both CPU cycles and wall-clock time (via features)
//! - Automatic CPU core pinning for stable measurements
//! - Randomized variant execution to avoid ordering bias
//! - All raw measurements preserved for external analysis

use std::hint::black_box;
use std::time::Duration;

pub use super::cpu_affinity::{pin_to_current_core, unpin, CpuPinGuard};
use super::bench::{shuffle, time_seed, to_nanos, Measurement};

// ============================================================================
// Configuration
// ============================================================================

/// CPU pinning strategy during measurements
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PinStrategy {
    /// Pin once before all measurements (minimal overhead)
    Global,
    /// Pin/unpin for each execution (current behavior, more accurate per-call)
    #[default]
    PerExecution,
}

/// Configuration for timing measurements
#[derive(Clone, Debug)]
pub struct TimingConfig {
    /// Number of samples to collect per variant (default: 30)
    pub runs_per_variant: usize,
    /// Number of warmup iterations before measurement (default: 10)
    pub warmup_iterations: usize,
    /// CPU pinning strategy (default: PerExecution)
    pub pin_strategy: PinStrategy,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            runs_per_variant: 30,
            warmup_iterations: 10,
            pin_strategy: PinStrategy::default(),
        }
    }
}

/// A variant to be measured
pub struct Variant<'a> {
    /// Unique name of the variant
    pub name: &'static str,
    /// Human-readable description
    pub description: &'static str,
    /// The function to benchmark - returns (measurement, optional result value)
    /// Timing happens inside the closure to eliminate Fn trait overhead.
    pub run: Box<dyn FnMut() -> (Measurement, Option<f64>) + 'a>,
}

/// Result from measuring a single variant
#[derive(Clone, Debug)]
pub struct VariantResult {
    /// Name of the variant
    pub name: String,
    /// Description of the variant
    pub description: String,
    /// Average measurement (as Duration for compatibility)
    pub avg_time: Duration,
    /// Precise average in nanoseconds/cycles as f64
    pub avg_nanos_f64: f64,
    /// Median measurement
    pub median_time: Duration,
    /// Minimum measurement
    pub min_time: Duration,
    /// Maximum measurement
    pub max_time: Duration,
    /// Standard deviation
    pub std_dev: Duration,
    /// Number of iterations performed
    pub iterations: usize,
    /// Sample result value (for verification) - only for algorithms that have meaningful results
    pub result_sample: Option<f64>,
}

/// Measure multiple variants with randomized execution order.
///
/// This is the main entry point for benchmarking. It:
/// 1. Warms up all variants
/// 2. Creates a randomized task schedule
/// 3. Measures each variant with CPU pinning
/// 4. Returns results for all variants
///
/// # Arguments
/// * `variants` - List of variants to measure
/// * `iterations` - Total iterations per variant (used for reporting)
/// * `config` - Timing configuration
///
/// # Returns
/// A vector of `VariantResult` for each variant
pub fn measure_variants(
    mut variants: Vec<Variant>,
    iterations: usize,
    config: &TimingConfig,
) -> Vec<VariantResult> {
    if variants.is_empty() {
        return Vec::new();
    }

    let samples = config.runs_per_variant;

    // Warmup all variants
    for variant in &mut variants {
        for _ in 0..config.warmup_iterations {
            black_box((variant.run)());
        }
    }

    // Create randomized task schedule: (variant_idx, sample_idx)
    let mut tasks: Vec<(usize, usize)> = (0..variants.len())
        .flat_map(|v| (0..samples).map(move |s| (v, s)))
        .collect();
    shuffle(&mut tasks, time_seed());

    // Storage for measurements (Vec for O(1) index access)
    let mut measurements: Vec<Vec<Measurement>> = (0..variants.len())
        .map(|_| Vec::with_capacity(samples))
        .collect();
    let mut result_samples: Vec<Option<f64>> = vec![None; variants.len()];

    let _global_pin = (config.pin_strategy == PinStrategy::Global).then(CpuPinGuard::new);

    for (variant_idx, _) in tasks {
        let variant = &mut variants[variant_idx];
        let _per_exec_pin = (config.pin_strategy == PinStrategy::PerExecution).then(CpuPinGuard::new);
        let (elapsed_time, result) = (variant.run)();

        measurements[variant_idx].push(elapsed_time);
        result_samples[variant_idx] = result;
    }

    variants.into_iter().enumerate().map(|(idx, variant)| {
            let times = std::mem::take(&mut measurements[idx]);
            let result_sample = result_samples[idx].take();
            compute_variant_result(variant.name, variant.description, times, iterations, result_sample)
        })
        .collect()
}

/// Compute statistics from raw measurements
fn compute_variant_result(
    name: &'static str,
    description: &'static str,
    measurements: Vec<Measurement>,
    iterations: usize,
    result_sample: Option<f64>,
) -> VariantResult {
    if measurements.is_empty() {
        return VariantResult {
            name: name.to_string(),
            description: description.to_string(),
            avg_time: Duration::ZERO,
            avg_nanos_f64: 0.0,
            median_time: Duration::ZERO,
            min_time: Duration::ZERO,
            max_time: Duration::ZERO,
            std_dev: Duration::ZERO,
            iterations,
            result_sample: None,
        };
    }

    let nanos: Vec<u64> = measurements.iter().map(|m| to_nanos(*m)).collect();

    let mut sorted = nanos.clone();
    sorted.sort();

    let min_ns = sorted[0];
    let max_ns = sorted[sorted.len() - 1];
    let median_ns = sorted[sorted.len() / 2];

    let sum: u64 = nanos.iter().sum();
    let avg_nanos_f64 = sum as f64 / nanos.len() as f64;
    let avg_ns = avg_nanos_f64 as u64;

    let variance: f64 = nanos
        .iter()
        .map(|&n| {
            let diff = n as f64 - avg_nanos_f64;
            diff * diff
        })
        .sum::<f64>()
        / (nanos.len() - 1).max(1) as f64;
    let std_dev_ns = variance.sqrt() as u64;

    VariantResult {
        name: name.to_string(),
        description: description.to_string(),
        avg_time: Duration::from_nanos(avg_ns),
        avg_nanos_f64,
        median_time: Duration::from_nanos(median_ns),
        min_time: Duration::from_nanos(min_ns),
        max_time: Duration::from_nanos(max_ns),
        std_dev: Duration::from_nanos(std_dev_ns),
        iterations,
        result_sample,
    }
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
    fn test_measure_variants_empty() {
        let results = measure_variants(vec![], 1000, &TimingConfig::default());
        assert!(results.is_empty());
    }

    #[test]
    fn test_measure_variants_single() {
        use crate::measure;

        let variants = vec![Variant {
            name: "test",
            description: "Test variant",
            run: Box::new(|| {
                let (elapsed, _) = measure!(42);
                (elapsed, Some(42.0))
            }),
        }];

        let config = TimingConfig {
            runs_per_variant: 5,
            warmup_iterations: 2,
            pin_strategy: PinStrategy::Global,
        };

        let results = measure_variants(variants, 100, &config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test");
        assert_eq!(results[0].result_sample, Some(42.0));
    }

    #[test]
    fn test_measure_variants_multiple() {
        use crate::measure;

        let variants = vec![
            Variant {
                name: "fast",
                description: "Fast variant",
                run: Box::new(|| {
                    let (elapsed, _) = measure!(1);
                    (elapsed, Some(1.0))
                }),
            },
            Variant {
                name: "slow",
                description: "Slow variant",
                run: Box::new(|| {
                    let (elapsed, _) = measure!(vec![0u8; 1000]);
                    (elapsed, Some(2.0))
                }),
            },
        ];

        let config = TimingConfig {
            runs_per_variant: 5,
            warmup_iterations: 2,
            pin_strategy: PinStrategy::PerExecution,
        };

        let results = measure_variants(variants, 100, &config);
        assert_eq!(results.len(), 2);

        let fast = results.iter().find(|r| r.name == "fast").unwrap();
        let slow = results.iter().find(|r| r.name == "slow").unwrap();

        assert_eq!(fast.result_sample, Some(1.0));
        assert_eq!(slow.result_sample, Some(2.0));
    }
}
