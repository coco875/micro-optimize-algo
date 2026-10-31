use crate::math::sin::SinVariant;
use crate::utils::math::{self, FloatErrorReport, MathDomain, MathParam};
use crate::utils::randomness::InputRng;
use crate::utils::runner::Config;
use crate::utils::scenario::{Scenario, ScenarioDefinition, ValidationCase};
use crate::utils::validation::Check;

const DOMAINS: &[MathDomain] = &[MathDomain::Angles, MathDomain::WideAngles];

#[crate::scenario(
    group = "sin",
    id = "random",
    name = "Sin",
    tags = "math, scalar-math, approximation"
)]
pub struct SinScenario<V> {
    inputs: Vec<f32>,
    marker: std::marker::PhantomData<V>,
}
impl<V: SinVariant> Scenario for SinScenario<V> {
    fn task(&mut self) {
        for &input in std::hint::black_box(&self.inputs) {
            let input = std::hint::black_box(input);
            std::hint::black_box(V::compute(input));
        }
    }
}
impl<V: SinVariant> ScenarioDefinition<V> for SinScenario<V> {
    type Param = MathParam;
    type Fixture = Vec<f32>;
    type Output = Vec<f32>;
    fn prepare(param: &MathParam, rng: &mut InputRng) -> Self::Fixture {
        math::unary_inputs(param, rng)
    }
    fn new(inputs: &Self::Fixture) -> Self {
        V::prepare();
        Self {
            inputs: inputs.clone(),
            marker: std::marker::PhantomData,
        }
    }
    fn generate_params(config: &Config) -> Vec<MathParam> {
        math::params(config, "sin", DOMAINS)
    }
    fn format_param(param: &MathParam) -> String {
        math::format_param(param)
    }
    fn validation_cases(_: &Config) -> Vec<ValidationCase<MathParam>> {
        math::validation_cases(DOMAINS)
    }
    fn validate(inputs: &Self::Fixture) -> Vec<f32> {
        V::prepare();
        inputs
            .iter()
            .copied()
            .map(|input| V::compute(input))
            .collect()
    }
    fn checks(_: &Self::Fixture) -> Vec<Box<dyn Check<Vec<f32>>>> {
        vec![Box::new(FloatErrorReport)]
    }
}
