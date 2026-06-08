//! # Else-If Chain vs Jump Table Comparison

pub mod code;
pub mod test;

pub use code::*;

use crate::scenario;
use crate::utils::{scenario::OtherMetric, Scenario, ScenarioDefinition};

/// Trait defining an else-if vs jumptable variant statically
pub trait ElseIfVsJumpTableVariant: Send + 'static {
    /// Variant name
    fn name() -> &'static str;
    /// Computation function
    fn compute(op: u8, val: u32) -> u32;
}

/// Scenario implementation for Else-If Chain vs Jump Table Comparison generic over variant V.
#[scenario(group = "elseif_vs_jumptable", tags = "control_flow, branch")]
pub struct ElseIfVsJumpTableScenario<V> {
    data: Vec<(u8, u32)>,
    result: Option<u32>,
    _marker: std::marker::PhantomData<V>,
}

impl<V: ElseIfVsJumpTableVariant> Scenario for ElseIfVsJumpTableScenario<V> {
    fn task(&mut self) {
        let mut last_result = 0u32;
        for &(op, val) in &self.data {
            last_result = std::hint::black_box(V::compute(
                std::hint::black_box(op),
                std::hint::black_box(val),
            ));
        }
        self.result = Some(last_result);
    }

    fn other_metric(&self) -> OtherMetric {
        self.result
            .map(|r| OtherMetric::Validation(r.to_string()))
            .unwrap_or(OtherMetric::None)
    }
}

impl<V: ElseIfVsJumpTableVariant> ScenarioDefinition<V> for ElseIfVsJumpTableScenario<V> {
    type Param = usize;

    fn new(size: usize, seed: u64) -> Self {
        let mut rng = crate::utils::bench::SeededRng::new(seed);
        let data = (0..size)
            .map(|_| {
                let op = rng.next_u32_range(8) as u8;
                let val = rng.next_u32_range(1000) + 1;
                (op, val)
            })
            .collect();
        Self {
            data,
            result: None,
            _marker: std::marker::PhantomData,
        }
    }

    fn generate_params(config: &crate::utils::runner::Config) -> Vec<usize> {
        if let Some(table) = config.get_table("elseif_vs_jumptable") {
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
