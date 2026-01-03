//! Shared benchmark utilities.
//!
//! Common functions used by all benchmark modules.

use std::time::Duration;

/// Calculate standard deviation from a list of durations
pub fn calculate_std_dev(times: &[Duration], mean: Duration) -> Duration {
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

/// Simple fast random shuffle using Fisher-Yates algorithm
pub fn shuffle<T>(slice: &mut [T], seed: u64) {
    let mut rng = SeededRng::new(seed);
    shuffle_with_rng(slice, &mut rng);
}

/// Shuffle using an existing RNG (allows sequential shuffles with state preserved)
pub fn shuffle_with_rng<T>(slice: &mut [T], rng: &mut SeededRng) {
    for i in (1..slice.len()).rev() {
        let j = (rng.next_u64() >> 33) as usize % (i + 1);
        slice.swap(i, j);
    }
}

/// Get a seed from current time for randomization
pub fn time_seed() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x12345678)
}

/// Compute timing statistics from a list of durations
pub fn compute_stats(times: &[Duration]) -> (Duration, Duration, Duration, Duration) {
    if times.is_empty() {
        return (Duration::ZERO, Duration::ZERO, Duration::ZERO, Duration::ZERO);
    }
    
    let min = *times.iter().min().unwrap();
    let max = *times.iter().max().unwrap();
    let total: Duration = times.iter().sum();
    let avg = total / times.len() as u32;
    let std_dev = calculate_std_dev(times, avg);
    
    (avg, min, max, std_dev)
}

/// Simple seeded PRNG for reproducible benchmarks
pub struct SeededRng {
    state: u64,
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    
    /// Generate next u64
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }
    
    /// Generate f32 in range [-1.0, 1.0)
    pub fn next_f32_range(&mut self) -> f32 {
        let n = self.next_u64();
        (n >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0
    }
    
    /// Generate u32 in range [0, max)
    pub fn next_u32_range(&mut self, max: u32) -> u32 {
        (self.next_u64() >> 32) as u32 % max
    }
}
