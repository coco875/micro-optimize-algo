//! Shared benchmark utilities.
//!
//! Common functions used by all benchmark modules.
//!
//! By default (`cpu_cycles` feature), measurements use CPU cycle counters
//! for precise micro-benchmarking. Use `--features use_time` or
//! `--no-default-features` to use wall-clock time instead.

use std::time::Duration;

// ============================================================================
// Measurement abstraction: cycles or time depending on feature flags
// ============================================================================
//
// Use CPU cycles if: cpu_cycles is enabled AND use_time is NOT enabled
// Use wall-clock time if: use_time is enabled OR cpu_cycles is disabled

/// Measurement value type - cycles (u64) or Duration depending on feature
#[cfg(all(feature = "cpu_cycles", not(feature = "use_time")))]
pub type Measurement = u64;

#[cfg(any(not(feature = "cpu_cycles"), feature = "use_time"))]
pub type Measurement = Duration;

/// Read current measurement (cycles or time)
#[cfg(all(feature = "cpu_cycles", not(feature = "use_time")))]
#[inline(always)]
pub fn now() -> Measurement {
    crate::utils::cycles::read_cycles()
}

#[cfg(any(not(feature = "cpu_cycles"), feature = "use_time"))]
#[inline(always)]
pub fn now() -> std::time::Instant {
    std::time::Instant::now()
}

/// Calculate elapsed measurement
#[cfg(all(feature = "cpu_cycles", not(feature = "use_time")))]
#[inline(always)]
pub fn elapsed(start: Measurement) -> Measurement {
    crate::utils::cycles::read_cycles().saturating_sub(start)
}

#[cfg(any(not(feature = "cpu_cycles"), feature = "use_time"))]
#[inline(always)]
pub fn elapsed(start: std::time::Instant) -> Measurement {
    start.elapsed()
}

/// Convert measurement to nanoseconds for display
#[cfg(all(feature = "cpu_cycles", not(feature = "use_time")))]
pub fn to_nanos(m: Measurement) -> u64 {
    // Return raw cycles
    m
}

#[cfg(any(not(feature = "cpu_cycles"), feature = "use_time"))]
pub fn to_nanos(m: Measurement) -> u64 {
    m.as_nanos() as u64
}

/// Get the measurement unit name
#[cfg(all(feature = "cpu_cycles", not(feature = "use_time")))]
pub const fn unit_name() -> &'static str {
    #[cfg(target_arch = "aarch64")]
    {
        "ticks"
    }
    #[cfg(target_arch = "x86_64")]
    {
        "cycles"
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        "units"
    }
}

#[cfg(any(not(feature = "cpu_cycles"), feature = "use_time"))]
pub const fn unit_name() -> &'static str {
    "ns"
}

/// Calculate standard deviation from a list of durations
pub fn calculate_std_dev(times: &[Duration], mean: Duration) -> Duration {
    if times.len() < 2 {
        return Duration::ZERO;
    }

    let mean_ns = mean.as_nanos() as f64;
    let variance: f64 = times
        .iter()
        .map(|t| {
            let diff = t.as_nanos() as f64 - mean_ns;
            diff * diff
        })
        .sum::<f64>()
        / (times.len() - 1) as f64;

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
        return (
            Duration::ZERO,
            Duration::ZERO,
            Duration::ZERO,
            Duration::ZERO,
        );
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

use crate::utils::timer::calculate_median;
use std::collections::HashMap;

/// Metadata for a variant being benchmarked
pub struct VariantTiming {
    pub name: String,
    pub description: String,
    pub times: Vec<Duration>,
    pub result_sample: f64,
}

/// Run a generic benchmark with randomized execution order.
///
/// # Type Parameters
/// * `V` - Variant type
/// * `F` - Execution function: takes variant and returns result as f64
///
/// # Arguments
/// * `variants` - List of (name, description, variant) tuples
/// * `samples_per_variant` - Number of samples to collect per variant
/// * `warmup_fn` - Warmup function called once per variant
/// * `execute_fn` - Function to execute and time (returns result)
pub fn run_generic_benchmark<V, W, E>(
    variants: &[(String, String, V)],
    samples_per_variant: usize,
    mut warmup_fn: W,
    mut execute_fn: E,
) -> Vec<VariantTiming>
where
    V: Clone,
    W: FnMut(&V),
    E: FnMut(&V) -> (Duration, f64),
{
    if variants.is_empty() {
        return Vec::new();
    }

    // Warmup
    for (_, _, variant) in variants {
        warmup_fn(variant);
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
    let mut result_samples: HashMap<usize, f64> = HashMap::new();

    // Execute
    for (variant_idx, _) in tasks {
        let (_, _, variant) = &variants[variant_idx];
        let (elapsed, result) = execute_fn(variant);
        timing_results.get_mut(&variant_idx).unwrap().push(elapsed);
        result_samples.insert(variant_idx, result);
    }

    // Collect results
    variants
        .iter()
        .enumerate()
        .map(|(idx, (name, description, _))| {
            let times = timing_results.remove(&idx).unwrap();
            VariantTiming {
                name: name.clone(),
                description: description.clone(),
                times,
                result_sample: *result_samples.get(&idx).unwrap_or(&0.0),
            }
        })
        .collect()
}

/// Convert VariantTiming to BenchmarkResult
pub fn timing_to_result(
    timing: VariantTiming,
    iterations: usize,
) -> crate::registry::BenchmarkResult {
    let (avg, min, max, std_dev) = compute_stats(&timing.times);
    crate::registry::BenchmarkResult {
        variant_name: timing.name,
        description: timing.description,
        avg_time: avg,
        median_time: calculate_median(&timing.times),
        min_time: min,
        max_time: max,
        std_dev,
        iterations,
        result_sample: timing.result_sample,
    }
}
