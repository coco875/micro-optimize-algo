//! Benchmark utilities for dot product.

use super::code::available_variants;
use std::time::{Duration, Instant};

/// Benchmark stats
pub struct BenchStats {
    pub avg: Duration,
    pub min: Duration,
    pub max: Duration,
}

/// Run a variant and return benchmark statistics
pub fn benchmark_variant(
    _name: &str,
    func: fn(&[f32], &[f32]) -> f32,
    a: &[f32],
    b: &[f32],
    total_iterations: usize,
) -> BenchStats {
    // Warmup
    for _ in 0..(total_iterations / 10).max(10) {
        let _ = func(a, b);
    }

    let samples = 30;
    let iter_per_sample = (total_iterations / samples).max(1);
    let mut sample_avgs = Vec::with_capacity(samples);

    for _ in 0..samples {
        let start = Instant::now();
        for _ in 0..iter_per_sample {
            let result = func(a, b);
            std::hint::black_box(result);
        }
        let elapsed = start.elapsed();
        sample_avgs.push(elapsed / iter_per_sample as u32);
    }

    let min = *sample_avgs.iter().min().unwrap();
    let max = *sample_avgs.iter().max().unwrap();
    let avg = sample_avgs.into_iter().sum::<Duration>() / samples as u32;

    BenchStats { avg, min, max }
}

/// Benchmark result for a variant
pub struct BenchResult {
    pub name: String,
    pub description: String,
    pub avg_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub result: f32,
    pub compiler: Option<String>,
}

/// Run all available variants and return benchmark results
pub fn run_all_benchmarks(a: &[f32], b: &[f32], iterations: usize) -> Vec<BenchResult> {
    let variants = available_variants();
    
    variants
        .into_iter()
        .map(|v| {
            let stats = benchmark_variant(v.name, v.function, a, b, iterations);
            let result = (v.function)(a, b);
            
            BenchResult {
                name: v.name.to_string(),
                description: v.description.to_string(),
                avg_time: stats.avg,
                min_time: stats.min,
                max_time: stats.max,
                result,
                compiler: v.compiler.map(|s| s.to_string()),
            }
        })
        .collect()
}
