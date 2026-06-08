//! # VTable Iteration Benchmark

pub mod code;
pub mod test;

pub use code::*;

use crate::scenario;
use crate::utils::{scenario::OtherMetric, Scenario, ScenarioDefinition};

/// Trait defining a vtable iteration variant statically
pub trait VTableIterationVariant: Send + 'static {
    /// Variant name
    fn name() -> &'static str;
    /// Creator function for the object pool
    fn create(size: usize, seed: u64) -> Box<dyn ObjectPool + Send>;
}

/// Scenario implementation for VTable Iteration Benchmark generic over variant V.
#[scenario(group = "vtable_iteration", tags = "control_flow, virtual")]
pub struct VTableIterationScenario<V> {
    pool: Option<Box<dyn ObjectPool + Send>>,
    result: Option<u64>,
    _marker: std::marker::PhantomData<V>,
}

impl<V: VTableIterationVariant> Scenario for VTableIterationScenario<V> {
    fn task(&mut self) {
        if let Some(pool) = &mut self.pool {
            self.result = Some(std::hint::black_box(pool.iterate()));
        }
    }

    fn other_metric(&self) -> OtherMetric {
        self.result
            .map(|r| OtherMetric::Validation(r.to_string()))
            .unwrap_or(OtherMetric::None)
    }
}

impl<V: VTableIterationVariant> ScenarioDefinition<V> for VTableIterationScenario<V> {
    type Param = usize;

    fn new(size: usize, seed: u64) -> Self {
        let pool = Some(V::create(size, seed));
        Self {
            pool,
            result: None,
            _marker: std::marker::PhantomData,
        }
    }

    fn generate_params(config: &crate::utils::runner::Config) -> Vec<usize> {
        if let Some(table) = config.get_table("vtable_iteration") {
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
