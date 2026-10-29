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

/// Only the intended workload executes inside the timer.
pub trait Scenario: Send {
    fn task(&mut self);
}

/// A workload definition, statically specialized for an implementation.
/// Fixture, Param and Output must be identical for every implementation of a scenario.
pub trait ScenarioDefinition<V>: Scenario {
    type Param: Clone + Send + Sync + 'static;
    type Fixture;
    type Output;
    fn prepare(param: &Self::Param, rng: &mut crate::utils::randomness::InputRng) -> Self::Fixture;
    fn new(fixture: &Self::Fixture) -> Self;
    fn generate_params(config: &crate::utils::runner::Config) -> Vec<Self::Param>;
    fn format_param(param: &Self::Param) -> String;
    fn validation_cases(config: &crate::utils::runner::Config) -> Vec<ValidationCase<Self::Param>>;
    /// Run once, retaining observations for all selected checks. Never called by benchmarking.
    fn validate(fixture: &Self::Fixture) -> Self::Output;
    fn checks(
        fixture: &Self::Fixture,
    ) -> Vec<Box<dyn crate::utils::validation::Check<Self::Output>>>;
}

pub struct ValidationCase<P> {
    pub id: String,
    pub param: P,
}

impl<P> ValidationCase<P> {
    pub fn new(id: impl Into<String>, param: P) -> Self {
        Self {
            id: id.into(),
            param,
        }
    }
}

pub fn size_params(config: &crate::utils::runner::Config, family: &str) -> Vec<usize> {
    config
        .get_table(family)
        .and_then(|table| table.get("sizes"))
        .and_then(toml::Value::as_array)
        .map(|sizes| {
            sizes
                .iter()
                .map(|size| {
                    size.as_integer()
                        .and_then(|n| usize::try_from(n).ok())
                        .expect("sizes must contain non-negative integers")
                })
                .collect()
        })
        .unwrap_or_else(|| vec![64, 256, 1024, 4096, 16384])
}
