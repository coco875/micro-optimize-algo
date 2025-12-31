//! Benchmarks for branch vs jumptable vs branchless comparison

use crate::registry::BenchmarkResult;
use super::code::{get_variants, DispatchFn};
use std::time::{Duration, Instant};
use std::hint::black_box;

/// Result from a single variant benchmark
pub struct VariantBenchResult {
    pub name: &'static str,
    pub description: &'static str,
    pub avg_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub std_dev: Duration,
    pub result: u32,
}

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

/// Calculate standard deviation from a list of durations
fn calculate_std_dev(times: &[Duration], mean: Duration) -> Duration {
    if times.len() < 2 {
        return Duration::ZERO;
    }
    
    let mean_ns = mean.as_nanos() as f64;
    let variance: f64 = times.iter()
        .map(|t| {
            let diff = t.as_nanos() as f64 - mean_ns;
            diff * diff
        })
        .sum::<f64>() / (times.len() - 1) as f64;
    
    let std_dev_ns = variance.sqrt();
    Duration::from_nanos(std_dev_ns as u64)
}

/// Run benchmark for a single function
fn benchmark_function(func: DispatchFn, data: &[(u8, u32)], iterations: usize) -> (Duration, Duration, Duration, Duration, u32) {
    // Warmup
    for &(op, val) in data.iter().take(100) {
        black_box(func(black_box(op), black_box(val)));
    }

    // Collect all timing measurements
    let mut times = Vec::with_capacity(iterations);
    let mut last_result = 0u32;
    
    for _ in 0..iterations {
        let start = Instant::now();
        for &(op, val) in data {
            last_result = black_box(func(black_box(op), black_box(val)));
        }
        times.push(start.elapsed());
    }
    
    // Calculate statistics
    let total: Duration = times.iter().sum();
    let avg = total / iterations as u32;
    let min_time = *times.iter().min().unwrap_or(&Duration::ZERO);
    let max_time = *times.iter().max().unwrap_or(&Duration::ZERO);
    let std_dev = calculate_std_dev(&times, avg);
    
    (avg, min_time, max_time, std_dev, last_result)
}

/// Run all benchmarks and return internal results
pub fn run_all_benchmarks(size: usize, iterations: usize) -> Vec<VariantBenchResult> {
    let data = generate_test_data(size);
    let variants = get_variants();
    
    variants.iter().map(|variant| {
        let (avg_time, min_time, max_time, std_dev, result) = benchmark_function(variant.func, &data, iterations);
        
        VariantBenchResult {
            name: variant.name,
            description: variant.description,
            avg_time,
            min_time,
            max_time,
            std_dev,
            result,
        }
    }).collect()
}

/// Run all benchmarks and return registry-compatible results
pub fn run_benchmarks(size: usize, iterations: usize) -> Vec<BenchmarkResult> {
    run_all_benchmarks(size, iterations)
        .into_iter()
        .map(|r| BenchmarkResult {
            variant_name: r.name.to_string(),
            description: r.description.to_string(),
            avg_time: r.avg_time,
            min_time: r.min_time,
            max_time: r.max_time,
            std_dev: r.std_dev,
            iterations,
            result_sample: r.result as f64,
            compiler: None,
        })
        .collect()
}
