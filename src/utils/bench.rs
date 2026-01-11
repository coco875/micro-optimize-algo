//! Measurement primitives and utilities.
//!
//! This module provides low-level measurement primitives that work with both
//! CPU cycles and wall-clock time depending on feature flags.
//!
//! By default (`cpu_cycles` feature), measurements use CPU cycle counters
//! for precise micro-benchmarking. Use `--features use_time` or
//! `--no-default-features` to use wall-clock time instead.

use std::time::Duration;

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

/// Format a Duration for display with appropriate unit based on features
pub fn format_measurement(d: std::time::Duration) -> String {
    #[cfg(all(feature = "cpu_cycles", not(feature = "use_time")))]
    {
        format!("{} {}", d.as_nanos(), unit_name())
    }

    #[cfg(any(not(feature = "cpu_cycles"), feature = "use_time"))]
    {
        format!("{:?}", d)
    }
}

/// Format a Duration with floating-point precision (for averages)
pub fn format_measurement_precise(nanos_f64: f64) -> String {
    #[cfg(all(feature = "cpu_cycles", not(feature = "use_time")))]
    {
        format!("{:.2} {}", nanos_f64, unit_name())
    }

    #[cfg(any(not(feature = "cpu_cycles"), feature = "use_time"))]
    {
        if nanos_f64 >= 1_000_000_000.0 {
            format!("{:.3}s", nanos_f64 / 1_000_000_000.0)
        } else if nanos_f64 >= 1_000_000.0 {
            format!("{:.3}ms", nanos_f64 / 1_000_000.0)
        } else if nanos_f64 >= 1_000.0 {
            format!("{:.3}µs", nanos_f64 / 1_000.0)
        } else {
            format!("{:.2}ns", nanos_f64)
        }
    }
}

/// Measures a single expression, returning (measurement, result).
/// Use inside variant closures to eliminate Fn trait overhead from timing.
///
/// # Example
/// ```ignore
/// use micro_optimize_algo::measure;
/// let (elapsed, result) = measure!(expensive_function(arg1, arg2));
/// ```
#[macro_export]
macro_rules! measure {
    ($expr:expr) => {{
        let start = $crate::utils::bench::now();
        let result = ::std::hint::black_box($expr);
        let elapsed = $crate::utils::bench::elapsed(start);
        (elapsed, result)
    }};
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
