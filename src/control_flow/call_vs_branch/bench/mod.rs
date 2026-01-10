//! Benchmarks for call vs branch comparison

use super::code::get_variants;
use crate::registry::BenchmarkResult;
use crate::utils::bench::{elapsed, now, run_generic_benchmark, timing_to_result};
use std::hint::black_box;

/// Generate test data - random values to stress branch prediction
fn generate_test_data(size: usize) -> Vec<u32> {
    let mut data = Vec::with_capacity(size);
    let mut seed: u64 = 0x12345678;
    for _ in 0..size {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        data.push((seed >> 32) as u32 % 512);
    }
    data
}

/// Run all benchmarks and return registry-compatible results
pub fn run_benchmarks(size: usize, iterations: usize) -> Vec<BenchmarkResult> {
    let data = generate_test_data(size);
    let variants = get_variants();
    if variants.is_empty() {
        return Vec::new();
    }

    let samples_per_variant = iterations.min(100);

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
            for &v in data.iter().take(100) {
                black_box(func(black_box(v)));
            }
        },
        |func| {
            // Execute and time
            let start = now();
            let mut last_result = 0u32;
            for &v in &data {
                last_result = black_box(func(black_box(v)));
            }
            let total = elapsed(start);

            #[cfg(all(feature = "cpu_cycles", not(feature = "use_time")))]
            let duration = std::time::Duration::from_nanos(crate::utils::bench::to_nanos(total));
            #[cfg(any(not(feature = "cpu_cycles"), feature = "use_time"))]
            let duration = total;

            (duration, last_result as f64)
        },
    );

    timings
        .into_iter()
        .map(|t| timing_to_result(t, iterations))
        .collect()
}
