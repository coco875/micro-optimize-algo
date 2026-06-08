pub mod code;
#[cfg(test)]
pub mod test;

pub use code::*;

use crate::scenario;
use crate::utils::{scenario::OtherMetric, Scenario, ScenarioDefinition};

/// Trait defining a xoroshiro variant statically
pub trait XoroshiroVariant: Send + 'static {
    /// Variant name
    fn name() -> &'static str;
    /// Computation step function
    fn compute(s0: &mut u64, s1: &mut u64) -> u64;
}

/// Scenario implementation for Xoroshiro128++ generic over variant V.
#[scenario(
    group = "xoroshiro",
    name = "xoroshiro128++",
    tags = "random, generator"
)]
pub struct XoroshiroScenario<V> {
    s0: u64,
    s1: u64,
    result: Option<u64>,
    _marker: std::marker::PhantomData<V>,
}

impl<V: XoroshiroVariant> Scenario for XoroshiroScenario<V> {
    fn task(&mut self) {
        self.result = Some(std::hint::black_box(V::compute(&mut self.s0, &mut self.s1)));
    }

    fn other_metric(&self) -> OtherMetric {
        self.result
            .map(|r| OtherMetric::Randomness(r.count_ones() as f64 / 64.0))
            .unwrap_or(OtherMetric::None)
    }
}

impl<V: XoroshiroVariant> ScenarioDefinition<V> for XoroshiroScenario<V> {
    type Param = ();

    fn new(_param: (), seed: u64) -> Self {
        let mut rng = crate::utils::bench::SeededRng::new(seed);
        let mut s0 = rng.next_u64();
        let mut s1 = rng.next_u64();
        // Avoid 0 seed state
        if s0 == 0 && s1 == 0 {
            s0 = 0x123456789ABCDEF0;
            s1 = 0xFEDCBA9876543210;
        }
        Self {
            s0,
            s1,
            result: None,
            _marker: std::marker::PhantomData,
        }
    }

    fn generate_params(_config: &crate::utils::runner::Config) -> Vec<()> {
        vec![()]
    }

    fn format_param(_param: &()) -> String {
        "N/A".to_string()
    }
}
