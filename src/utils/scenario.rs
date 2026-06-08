//! Scenario-based timing and benchmarking.
//!
//! This module defines the atomic, seed-initializable `Scenario` trait
//! to allow flexible, customizable benchmarks for various algorithms.

/// A struct that measures CPU cycles (or falling back to wall-clock nanoseconds
/// depending on available features) for precise micro-benchmarking.
pub struct CpuTimer {
    #[cfg(all(feature = "cpu_cycles", not(feature = "use_time")))]
    start: u64,
    #[cfg(any(not(feature = "cpu_cycles"), feature = "use_time"))]
    start: std::time::Instant,
}

impl CpuTimer {
    /// Creates a new CPU timer capturing the current time/cycles.
    #[inline]
    pub fn new() -> Self {
        Self {
            #[cfg(all(feature = "cpu_cycles", not(feature = "use_time")))]
            start: crate::utils::cycles::read_cycles(),
            #[cfg(any(not(feature = "cpu_cycles"), feature = "use_time"))]
            start: std::time::Instant::now(),
        }
    }

    /// Returns the elapsed cycles (or nanoseconds) since the timer was created.
    #[inline]
    pub fn elapsed_cycles(&self) -> u64 {
        #[cfg(all(feature = "cpu_cycles", not(feature = "use_time")))]
        {
            crate::utils::cycles::read_cycles().saturating_sub(self.start)
        }
        #[cfg(any(not(feature = "cpu_cycles"), feature = "use_time"))]
        {
            self.start.elapsed().as_nanos() as u64
        }
    }
}

impl Default for CpuTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OtherMetric {
    /// A value used to verify correctness across variants (must match exactly)
    Validation(String),
    /// A precision score/measurement (e.g. float value to compare/display)
    Precision(f64),
    /// A randomness score (e.g. fraction of set bits, entropy)
    Randomness(f64),
    /// No extra metric
    None,
}

impl std::fmt::Display for OtherMetric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OtherMetric::Validation(val) => write!(f, "Validation: {}", val),
            OtherMetric::Precision(p) => write!(f, "Precision: {:.6e}", p),
            OtherMetric::Randomness(r) => write!(f, "Randomness: {:.4}", r),
            OtherMetric::None => write!(f, "None"),
        }
    }
}

/// A scenario represents an atomic task to be measured.
pub trait Scenario: Send {
    /// The actual task to measure.
    fn task(&mut self);

    /// Optional other metric (validation, precision, randomness, or none).
    fn other_metric(&self) -> OtherMetric;
}

/// Trait implemented by scenario definitions.
pub trait ScenarioDefinition<V>: Scenario {
    type Param: Clone + Send + Sync + 'static;
    fn new(param: Self::Param, seed: u64) -> Self;
    fn generate_params(config: &crate::utils::runner::Config) -> Vec<Self::Param>;
    fn format_param(param: &Self::Param) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_timer() {
        let timer = CpuTimer::new();
        let _ = timer.elapsed_cycles();
    }
}
