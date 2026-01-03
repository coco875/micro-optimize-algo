//! Benchmark utilities for dot product.

use super::code::available_variants;
use crate::utils::bench::{compute_stats, shuffle, time_seed};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Benchmark result for a variant
pub struct BenchResult {
    pub name: String,
    pub description: String,
    pub avg_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub std_dev: Duration,
    pub result: f32,
    pub compiler: Option<String>,
}

/// Run all available variants and return benchmark results
pub fn run_all_benchmarks(a: &[f32], b: &[f32], iterations: usize) -> Vec<BenchResult> {
    let variants = available_variants();
    if variants.is_empty() {
        return Vec::new();
    }
    
    let samples_per_variant = 30;
    let iter_per_sample = (iterations / samples_per_variant).max(1);
    
    // Warmup
    for v in &variants {
        for _ in 0..(iterations / 10).max(10) {
            std::hint::black_box((v.function)(a, b));
        }
    }
    
    // Create and shuffle tasks
    let mut tasks: Vec<(usize, usize)> = (0..variants.len())
        .flat_map(|v| (0..samples_per_variant).map(move |s| (v, s)))
        .collect();
    shuffle(&mut tasks, time_seed());
    
    // Storage
    let mut timing_results: HashMap<usize, Vec<Duration>> = (0..variants.len())
        .map(|i| (i, Vec::with_capacity(samples_per_variant)))
        .collect();
    
    // Execute
    for (variant_idx, _) in tasks {
        let func = variants[variant_idx].function;
        
        let start = Instant::now();
        for _ in 0..iter_per_sample {
            std::hint::black_box(func(a, b));
        }
        let sample_avg = start.elapsed() / iter_per_sample as u32;
        timing_results.get_mut(&variant_idx).unwrap().push(sample_avg);
    }
    
    // Collect results
    variants.into_iter().enumerate().map(|(idx, v)| {
        let times = timing_results.get(&idx).unwrap();
        let (avg, min, max, std_dev) = compute_stats(times);
        
        BenchResult {
            name: v.name.to_string(),
            description: v.description.to_string(),
            avg_time: avg,
            min_time: min,
            max_time: max,
            std_dev,
            result: (v.function)(a, b),
            compiler: v.compiler.map(|s| s.to_string()),
        }
    }).collect()
}
