//! # Gradient Dot Product Algorithm
//!
//! Computes the dot product of Perlin gradient vectors and position vectors over slices of inputs.

pub mod code;
pub mod test;

pub use code::*;

use crate::scenario;
use crate::utils::{scenario::OtherMetric, Scenario, ScenarioDefinition};

/// Trait defining a gradient dot product variant statically
pub trait GradDotVariant: Send + 'static {
    /// Variant name
    fn name() -> &'static str;
    /// Computation function
    fn compute(hashes: &[usize], x: &[f64], y: &[f64], z: &[f64]) -> f64;
}

/// Scenario implementation for the gradient dot product algorithm generic over variant V.
#[scenario(group = "grad_dot", tags = "math, noise, 3d noise")]
pub struct GradDotScenario<V> {
    hashes: Vec<usize>,
    x: Vec<f64>,
    y: Vec<f64>,
    z: Vec<f64>,
    result: Option<f64>,
    _marker: std::marker::PhantomData<V>,
}

impl<V: GradDotVariant> Scenario for GradDotScenario<V> {
    fn task(&mut self) {
        self.result = Some(std::hint::black_box(V::compute(
            &self.hashes,
            &self.x,
            &self.y,
            &self.z,
        )));
    }

    fn other_metric(&self) -> OtherMetric {
        self.result
            .map(OtherMetric::Precision)
            .unwrap_or(OtherMetric::None)
    }
}

impl<V: GradDotVariant> ScenarioDefinition<V> for GradDotScenario<V> {
    type Param = usize;

    fn new(size: usize, seed: u64) -> Self {
        let mut rng = crate::utils::bench::SeededRng::new(seed);
        let hashes = (0..size)
            .map(|_| rng.next_u32_range(256) as usize)
            .collect();
        let x = (0..size).map(|_| rng.next_f32_range() as f64).collect();
        let y = (0..size).map(|_| rng.next_f32_range() as f64).collect();
        let z = (0..size).map(|_| rng.next_f32_range() as f64).collect();
        Self {
            hashes,
            x,
            y,
            z,
            result: None,
            _marker: std::marker::PhantomData,
        }
    }

    fn generate_params(config: &crate::utils::runner::Config) -> Vec<usize> {
        if let Some(table) = config.get_table("grad_dot") {
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
