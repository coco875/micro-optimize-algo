use super::code::available_variants;
use std::time::{Duration, Instant};
use std::hint::black_box;

pub struct BenchStats {
    pub avg: Duration,
    pub min: Duration,
    pub max: Duration,
    pub std_dev: Duration,
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

pub fn benchmark_variant(
    func: fn(&mut u64, &mut u64) -> u64,
    size: usize,
    total_iterations: usize,
) -> BenchStats {
    // Warmup
    // We must warm up using the same access pattern (inner loop of 'size')
    let mut s0 = 123456789;
    let mut s1 = 987654321;
    let warmup_batches = (total_iterations / 10).max(5).min(100); // Don't warmup too long, but do enough
    
    for _ in 0..warmup_batches {
        for _ in 0..size {
             black_box(func(&mut s0, &mut s1));
        }
    }

    let samples = 30;
    let iter_per_sample = (total_iterations / samples).max(1);
    let mut sample_avgs = Vec::with_capacity(samples);

    // Reset seed for consistency (though performance shouldn't vary with seed for Xoroshiro)
    s0 = 123456789;
    s1 = 987654321;

    for _ in 0..samples {
        let start = Instant::now();
        for _ in 0..iter_per_sample {
            // Inner loop: generate 'size' items
            for _ in 0..size {
                let res = func(&mut s0, &mut s1);
                black_box(res);
            }
        }
        let elapsed = start.elapsed();
        sample_avgs.push(elapsed / iter_per_sample as u32);
    }

    let min = *sample_avgs.iter().min().unwrap();
    let max = *sample_avgs.iter().max().unwrap();
    let avg = sample_avgs.iter().copied().sum::<Duration>() / samples as u32;
    let std_dev = calculate_std_dev(&sample_avgs, avg);

    BenchStats { avg, min, max, std_dev }
}

pub struct BenchResult {
    pub name: String,
    pub description: String,
    pub avg_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub std_dev: Duration,
    pub result: u64, // Just the last random number generated
    pub compiler: Option<String>,
}

pub fn run_all_benchmarks(size: usize, iterations: usize) -> Vec<BenchResult> {
    let variants = available_variants();
    
    variants.into_iter().map(|v| {
        let stats = benchmark_variant(v.function, size, iterations);
        
        // Compute a "result" sample (simple run)
        let mut s0 = 123456789;
        let mut s1 = 987654321;
        let result = (v.function)(&mut s0, &mut s1);
        
        BenchResult {
            name: v.name.to_string(),
            description: v.description.to_string(),
            avg_time: stats.avg,
            min_time: stats.min,
            max_time: stats.max,
            std_dev: stats.std_dev,
            result,
            compiler: v.compiler.map(|s| s.to_string()),
        }
    }).collect()
}
