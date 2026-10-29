use crate::control_flow::call_vs_branch::CallVsBranchVariant;
use crate::utils::randomness::InputRng;
use crate::utils::scenario::{size_params, Scenario, ScenarioDefinition, ValidationCase};
use crate::utils::validation::{Check, Exact};

#[crate::scenario(
    group = "call_vs_branch",
    id = "random",
    name = "CallVsBranch",
    tags = "control_flow"
)]
pub struct CallVsBranchScenario<V> {
    data: Vec<u32>,
    marker: std::marker::PhantomData<V>,
}
impl<V: CallVsBranchVariant> Scenario for CallVsBranchScenario<V> {
    fn task(&mut self) {
        for &value in &self.data {
            let value = std::hint::black_box(value);
            std::hint::black_box(V::compute(value));
        }
    }
}
impl<V: CallVsBranchVariant> ScenarioDefinition<V> for CallVsBranchScenario<V> {
    type Param = usize;
    type Fixture = Vec<u32>;
    type Output = Vec<u32>;
    fn prepare(size: &usize, rng: &mut InputRng) -> Self::Fixture {
        (0..*size).map(|_| rng.next_u32_range(512)).collect()
    }
    fn new(data: &Self::Fixture) -> Self {
        Self {
            data: data.clone(),
            marker: std::marker::PhantomData,
        }
    }
    fn generate_params(config: &crate::utils::runner::Config) -> Vec<usize> {
        size_params(config, "call_vs_branch")
    }
    fn format_param(param: &usize) -> String {
        param.to_string()
    }
    fn validation_cases(_: &crate::utils::runner::Config) -> Vec<ValidationCase<usize>> {
        [0, 1, 8, 512, 4096]
            .into_iter()
            .map(|n| ValidationCase::new(n.to_string(), n))
            .collect()
    }
    fn validate(data: &Self::Fixture) -> Vec<u32> {
        data.iter().map(|&value| V::compute(value)).collect()
    }
    fn checks(_: &Self::Fixture) -> Vec<Box<dyn Check<Vec<u32>>>> {
        vec![Box::new(Exact)]
    }
}
