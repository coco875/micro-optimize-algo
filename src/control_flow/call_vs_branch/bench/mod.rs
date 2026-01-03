//! Benchmarks for call vs branch comparison

use crate::registry::BenchmarkResult;
use crate::utils::bench::{shuffle, time_seed, compute_stats};
use super::code::get_variants;
use std::time::{Duration, Instant};
use std::hint::black_box;
use std::collections::HashMap;

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
    
    // Warmup
    for variant in &variants {
        for &v in data.iter().take(100) {
            black_box((variant.func)(black_box(v)));
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
    let mut last_results: HashMap<usize, u32> = HashMap::new();
    
    // Execute
    for (variant_idx, _) in tasks {
        let func = variants[variant_idx].func;
        
        let start = Instant::now();
        let mut last_result = 0u32;
        for &v in &data {
            last_result = black_box(func(black_box(v)));
        }
        timing_results.get_mut(&variant_idx).unwrap().push(start.elapsed());
        last_results.insert(variant_idx, last_result);
    }
    
    // Collect results
    variants.iter().enumerate().map(|(idx, variant)| {
        let times = timing_results.get(&idx).unwrap();
        let (avg, min, max, std_dev) = compute_stats(times);
        
        BenchmarkResult {
            variant_name: variant.name.to_string(),
            description: variant.description.to_string(),
            avg_time: avg,
            min_time: min,
            max_time: max,
            std_dev,
            iterations,
            result_sample: *last_results.get(&idx).unwrap_or(&0) as f64,
            compiler: None,
        }
    }).collect()
}
