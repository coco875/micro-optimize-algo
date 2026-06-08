//! Unified timing configuration.

/// CPU pinning strategy during measurements
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PinStrategy {
    /// Pin once before all measurements (minimal overhead)
    Global,
    /// Pin/unpin for each execution (more accurate per-call)
    #[default]
    PerExecution,
}

/// Configuration for timing measurements
#[derive(Clone, Debug)]
pub struct TimingConfig {
    /// Number of samples to collect per variant (default: 30)
    pub runs_per_variant: usize,
    /// Number of warmup iterations before measurement (default: 10)
    pub warmup_iterations: usize,
    /// CPU pinning strategy (default: PerExecution)
    pub pin_strategy: PinStrategy,
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            runs_per_variant: 30,
            warmup_iterations: 10,
            pin_strategy: PinStrategy::default(),
        }
    }
}

pub fn calculate_median(times: &[std::time::Duration]) -> std::time::Duration {
    if times.is_empty() {
        return std::time::Duration::ZERO;
    }
    let mut sorted: Vec<_> = times.to_vec();
    sorted.sort();
    sorted[sorted.len() / 2]
}
