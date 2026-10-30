//! Unified timing configuration.

pub const DEFAULT_WARMUP_ITERATIONS: usize = 10;

/// CPU pinning strategy during measurements
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
pub enum PinStrategy {
    /// Pin once before all measurements (minimal overhead)
    #[serde(rename = "global")]
    Global,
    /// Pin to the session's chosen core for each execution.
    #[default]
    #[serde(rename = "per-call", alias = "per-execution")]
    PerExecution,
}

impl std::str::FromStr for PinStrategy {
    type Err = &'static str;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "global" => Ok(Self::Global),
            "per-call" | "per-execution" => Ok(Self::PerExecution),
            _ => Err("pin strategy must be 'global' or 'per-call'"),
        }
    }
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
            warmup_iterations: DEFAULT_WARMUP_ITERATIONS,
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
