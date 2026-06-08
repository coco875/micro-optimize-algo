//! # Dot Product Algorithm
//!
//! The dot product computes the sum of products of corresponding elements in two vectors.

pub mod code;
pub mod test;

pub use code::*;

use crate::scenario;
use crate::utils::{scenario::OtherMetric, Scenario, ScenarioDefinition};

/// Trait defining a dot product variant statically
pub trait DotProductVariant: Send + 'static {
    /// Variant name
    fn name() -> &'static str;
    /// Computation function
    fn compute(a: &[f32], b: &[f32]) -> f32;
}

/// Scenario implementation for the dot product algorithm generic over variant V.
#[scenario(group = "dot_product", tags = "math, vector, 1d")]
pub struct DotProductScenario<V> {
    a: Vec<f32>,
    b: Vec<f32>,
    result: Option<f32>,
    _marker: std::marker::PhantomData<V>,
}

impl<V: DotProductVariant> Scenario for DotProductScenario<V> {
    fn task(&mut self) {
        self.result = Some(std::hint::black_box(V::compute(&self.a, &self.b)));
    }

    fn other_metric(&self) -> OtherMetric {
        self.result
            .map(|r| OtherMetric::Precision(r as f64))
            .unwrap_or(OtherMetric::None)
    }
}

impl<V: DotProductVariant> ScenarioDefinition<V> for DotProductScenario<V> {
    type Param = usize;

    fn new(size: usize, seed: u64) -> Self {
        let mut rng = crate::utils::bench::SeededRng::new(seed);
        let a = (0..size).map(|_| rng.next_f32_range()).collect();
        let b = (0..size).map(|_| rng.next_f32_range()).collect();
        Self {
            a,
            b,
            result: None,
            _marker: std::marker::PhantomData,
        }
    }

    fn generate_params(config: &crate::utils::runner::Config) -> Vec<usize> {
        if let Some(table) = config.get_table("dot_product") {
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
