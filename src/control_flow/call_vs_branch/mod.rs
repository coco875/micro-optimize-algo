//! # Call vs Branch Comparison

pub mod code;
pub mod test;

pub use code::*;

use crate::scenario;
use crate::utils::{scenario::OtherMetric, Scenario, ScenarioDefinition};

/// Trait defining a call vs branch variant statically
pub trait CallVsBranchVariant: Send + 'static {
    /// Variant name
    fn name() -> &'static str;
    /// Computation function
    fn compute(val: u32) -> u32;
}

/// Scenario implementation for Call vs Branch Comparison generic over variant V.
#[scenario(group = "call_vs_branch", tags = "control_flow")]
pub struct CallVsBranchScenario<V> {
    data: Vec<u32>,
    result: Option<u32>,
    _marker: std::marker::PhantomData<V>,
}

impl<V: CallVsBranchVariant> Scenario for CallVsBranchScenario<V> {
    fn task(&mut self) {
        let mut last_result = 0u32;
        for &val in &self.data {
            last_result = std::hint::black_box(V::compute(std::hint::black_box(val)));
        }
        self.result = Some(last_result);
    }

    fn other_metric(&self) -> OtherMetric {
        self.result
            .map(|r| OtherMetric::Validation(r.to_string()))
            .unwrap_or(OtherMetric::None)
    }
}

impl<V: CallVsBranchVariant> ScenarioDefinition<V> for CallVsBranchScenario<V> {
    type Param = usize;

    fn new(size: usize, seed: u64) -> Self {
        let mut rng = crate::utils::bench::SeededRng::new(seed);
        let data = (0..size).map(|_| rng.next_u32_range(512)).collect();
        Self {
            data,
            result: None,
            _marker: std::marker::PhantomData,
        }
    }

    fn generate_params(config: &crate::utils::runner::Config) -> Vec<usize> {
        if let Some(table) = config.get_table("call_vs_branch") {
            if let Some(sizes_val) = table.get("sizes") {
                if let Some(arr) = sizes_val.as_array() {
                    return arr
                        .iter()
                        .filter_map(|v| v.as_integer().map(|i| i as usize))
                        .collect();
                }
            }
        }
        vec![64, 256, 1024, 4096, 16384]
    }

    fn format_param(param: &usize) -> String {
        param.to_string()
    }
}
