//! Benchmark utilities for xoroshiro PRNG.

use super::code::available_variants;
use crate::utils::bench::{shuffle, time_seed, compute_stats};
use std::time::{Duration, Instant};
use std::hint::black_box;
use std::collections::HashMap;

pub struct BenchResult {
    pub name: String,
    pub description: String,
    pub avg_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub std_dev: Duration,
    pub result: u64,
    pub compiler: Option<String>,
}

/// Run all available variants and return benchmark results
pub fn run_all_benchmarks(size: usize, iterations: usize) -> Vec<BenchResult> {
    let variants = available_variants();
    if variants.is_empty() {
        return Vec::new();
    }
    
    let samples_per_variant = 30;
    let iter_per_sample = (iterations / samples_per_variant).max(1);
    
    // Warmup
    for v in &variants {
        let mut s0: u64 = 123456789;
        let mut s1: u64 = 987654321;
        for _ in 0..(iterations / 10).max(5).min(100) {
            for _ in 0..size {
                black_box((v.function)(&mut s0, &mut s1));
            }
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
    
    // Per-variant RNG state
    let mut rng_states: Vec<(u64, u64)> = vec![(123456789, 987654321); variants.len()];
    
    // Execute
    for (variant_idx, _) in tasks {
        let func = variants[variant_idx].function;
        let (ref mut s0, ref mut s1) = rng_states[variant_idx];
        
        let start = Instant::now();
        for _ in 0..iter_per_sample {
            for _ in 0..size {
                black_box(func(s0, s1));
            }
        }
        let sample_avg = start.elapsed() / iter_per_sample as u32;
        timing_results.get_mut(&variant_idx).unwrap().push(sample_avg);
    }
    
    // Collect results
    variants.into_iter().enumerate().map(|(idx, v)| {
        let times = timing_results.get(&idx).unwrap();
        let (avg, min, max, std_dev) = compute_stats(times);
        
        let mut s0: u64 = 123456789;
        let mut s1: u64 = 987654321;
        let result = (v.function)(&mut s0, &mut s1);
        
        BenchResult {
            name: v.name.to_string(),
            description: v.description.to_string(),
            avg_time: avg,
            min_time: min,
            max_time: max,
            std_dev,
            result,
            compiler: v.compiler.map(|s| s.to_string()),
        }
    }).collect()
}
