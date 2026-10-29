use crate::control_flow::vtable_iteration::{code::ObjectPool, VTableIterationVariant};
use crate::utils::randomness::InputRng;
use crate::utils::scenario::{size_params, Scenario, ScenarioDefinition, ValidationCase};
use crate::utils::validation::{Check, Exact};

#[crate::scenario(
    group = "vtable_iteration",
    id = "mixed",
    name = "VTableIteration",
    tags = "control_flow, virtual"
)]
pub struct VTableIterationScenario<V> {
    pool: Box<dyn ObjectPool + Send>,
    marker: std::marker::PhantomData<V>,
}
impl<V: VTableIterationVariant> Scenario for VTableIterationScenario<V> {
    fn task(&mut self) {
        std::hint::black_box(self.pool.iterate());
    }
}
impl<V: VTableIterationVariant> ScenarioDefinition<V> for VTableIterationScenario<V> {
    type Param = usize;
    type Fixture = Vec<(u8, u8)>;
    type Output = Vec<u64>;
    fn prepare(size: &usize, rng: &mut InputRng) -> Self::Fixture {
        (0..*size)
            .map(|i| (rng.next_u32_range(7) as u8, i as u8))
            .collect()
    }
    fn new(objects: &Self::Fixture) -> Self {
        Self {
            pool: V::create(objects),
            marker: std::marker::PhantomData,
        }
    }
    fn generate_params(config: &crate::utils::runner::Config) -> Vec<usize> {
        size_params(config, "vtable_iteration")
    }
    fn format_param(param: &usize) -> String {
        param.to_string()
    }
    fn validation_cases(_: &crate::utils::runner::Config) -> Vec<ValidationCase<usize>> {
        [1, 7, 64, 257, 1024]
            .into_iter()
            .map(|n| ValidationCase::new(n.to_string(), n))
            .collect()
    }
    fn validate(objects: &Self::Fixture) -> Vec<u64> {
        let mut pool = V::create(objects);
        (0..4).map(|_| pool.iterate()).collect()
    }
    fn checks(_: &Self::Fixture) -> Vec<Box<dyn Check<Vec<u64>>>> {
        vec![Box::new(Exact)]
    }
}
