//! Benchmark utilities for dot product.

use super::code::available_variants;
use crate::registry::BenchmarkResult;
use crate::utils::bench::{elapsed, now, run_generic_benchmark, timing_to_result};
use std::hint::black_box;

/// Run all available variants and return benchmark results
pub fn run_all_benchmarks(a: &[f32], b: &[f32], iterations: usize) -> Vec<BenchmarkResult> {
    let variants = available_variants();
    if variants.is_empty() {
        return Vec::new();
    }

    let samples_per_variant = 30;
    let iter_per_sample = (iterations / samples_per_variant).max(1);

    // Convert to generic format
    let variant_data: Vec<_> = variants
        .iter()
        .map(|v| (v.name.to_string(), v.description.to_string(), v.function))
        .collect();

    let timings = run_generic_benchmark(
        &variant_data,
        samples_per_variant,
        |func| {
            // Warmup
            for _ in 0..(iterations / 10).max(10) {
                black_box(func(a, b));
            }
        },
        |func| {
            // Execute and time using the abstracted measurement
            let start = now();
            for _ in 0..iter_per_sample {
                black_box(func(a, b));
            }
            let total = elapsed(start);

            // Convert to Duration for now (will be refactored when VariantTiming supports Measurement)
            #[cfg(feature = "cpu_cycles")]
            let sample_avg = std::time::Duration::from_nanos(
                crate::utils::bench::to_nanos(total) / iter_per_sample as u64,
            );
            #[cfg(not(feature = "cpu_cycles"))]
            let sample_avg = total / iter_per_sample as u32;

            let result = func(a, b) as f64;
            (sample_avg, result)
        },
    );

    timings
        .into_iter()
        .map(|t| timing_to_result(t, iterations))
        .collect()
}
