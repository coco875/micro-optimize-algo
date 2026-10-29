use super::random::{checks, DotFixture, DotProductScenario};
use crate::math::dot_product::DotProductVariant;
use crate::utils::randomness::InputRng;
use crate::utils::scenario::{size_params, Scenario, ScenarioDefinition, ValidationCase};
use crate::utils::validation::Check;

#[crate::scenario(
    group = "dot_product",
    id = "cancellation",
    name = "Cancellation",
    tags = "math, vector, cancellation"
)]
pub struct CancellationScenario<V> {
    workload: DotProductScenario<V>,
}
impl<V: DotProductVariant> Scenario for CancellationScenario<V> {
    fn task(&mut self) {
        self.workload.task();
    }
}
impl<V: DotProductVariant> ScenarioDefinition<V> for CancellationScenario<V> {
    type Param = usize;
    type Fixture = DotFixture;
    type Output = f64;
    fn prepare(size: &usize, rng: &mut InputRng) -> DotFixture {
        let a = (0..*size)
            .map(|i| match i % 4 {
                0 => 128.0,
                1 => rng.next_f32_range() * 0.001,
                2 => -128.0,
                _ => rng.next_f32_range() * 0.001,
            })
            .collect();
        DotFixture {
            a,
            b: vec![1.0; *size],
        }
    }
    fn new(fixture: &DotFixture) -> Self {
        Self {
            workload: DotProductScenario::new(fixture),
        }
    }
    fn generate_params(config: &crate::utils::runner::Config) -> Vec<usize> {
        size_params(config, "dot_product")
    }
    fn format_param(param: &usize) -> String {
        param.to_string()
    }
    fn validation_cases(_: &crate::utils::runner::Config) -> Vec<ValidationCase<usize>> {
        [4, 7, 17, 64]
            .into_iter()
            .map(|n| ValidationCase::new(n.to_string(), n))
            .collect()
    }
    fn validate(fixture: &DotFixture) -> f64 {
        DotProductScenario::<V>::validate(fixture)
    }
    fn checks(fixture: &DotFixture) -> Vec<Box<dyn Check<f64>>> {
        checks(fixture)
    }
}
