//! Benchmarks for branch vs jumptable vs branchless comparison

use super::code::get_variants;
use crate::registry::BenchmarkResult;
use crate::utils::bench::{run_generic_benchmark, timing_to_result};
use std::hint::black_box;
use std::time::Instant;

/// Generate test data - random opcodes (0-7) and values
fn generate_test_data(size: usize) -> Vec<(u8, u32)> {
    let mut data = Vec::with_capacity(size);
    let mut seed: u64 = 0x87654321;
    for _ in 0..size {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let opcode = ((seed >> 32) % 8) as u8;
        let value = ((seed >> 40) % 1000) as u32 + 1;
        data.push((opcode, value));
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
        .map(|v| {
            (
                v.name.to_string(),
                v.description.to_string(),
                None::<String>, // No compiler for these variants
                v.func,
            )
        })
        .collect();

    let timings = run_generic_benchmark(
        &variant_data,
        samples_per_variant,
        |func| {
            // Warmup
            for &(op, val) in data.iter().take(100) {
                black_box(func(black_box(op), black_box(val)));
            }
        },
        |func| {
            // Execute and time
            let start = Instant::now();
            let mut last_result = 0u32;
            for &(op, val) in &data {
                last_result = black_box(func(black_box(op), black_box(val)));
            }
            (start.elapsed(), last_result as f64)
        },
    );

    timings
        .into_iter()
        .map(|t| timing_to_result(t, iterations))
        .collect()
}
