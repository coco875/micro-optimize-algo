use crate::random::xoroshiro::XoroshiroVariant;
use crate::utils::randomness::InputRng;
use crate::utils::scenario::{Scenario, ScenarioDefinition, ValidationCase};
use crate::utils::validation::{ByteEntropy, Check, Exact, Project};

pub struct RngFixture {
    initial: (u64, u64),
    count: usize,
}
#[derive(Debug, PartialEq)]
pub struct SequenceOutput {
    values: Vec<u64>,
    final_state: (u64, u64),
}

#[crate::scenario(
    group = "xoroshiro",
    id = "sequence",
    name = "xoroshiro128++",
    tags = "random, generator"
)]
pub struct XoroshiroScenario<V> {
    state: (u64, u64),
    count: usize,
    marker: std::marker::PhantomData<V>,
}
impl<V: XoroshiroVariant> Scenario for XoroshiroScenario<V> {
    fn task(&mut self) {
        for _ in 0..self.count {
            std::hint::black_box(V::compute(&mut self.state.0, &mut self.state.1));
        }
    }
}
impl<V: XoroshiroVariant> ScenarioDefinition<V> for XoroshiroScenario<V> {
    type Param = usize;
    type Fixture = RngFixture;
    type Output = SequenceOutput;
    fn prepare(count: &usize, rng: &mut InputRng) -> RngFixture {
        let mut initial = (rng.next_u64(), rng.next_u64());
        if initial == (0, 0) {
            initial.0 = 1;
        }
        RngFixture {
            initial,
            count: *count,
        }
    }
    fn new(f: &RngFixture) -> Self {
        Self {
            state: f.initial,
            count: f.count,
            marker: std::marker::PhantomData,
        }
    }
    fn generate_params(_: &crate::utils::runner::Config) -> Vec<usize> {
        vec![1]
    }
    fn format_param(_: &usize) -> String {
        "N/A".into()
    }
    fn validation_cases(_: &crate::utils::runner::Config) -> Vec<ValidationCase<usize>> {
        vec![ValidationCase::new("sequence-65536", 65536)]
    }
    fn validate(f: &RngFixture) -> SequenceOutput {
        let mut state = f.initial;
        let values = (0..f.count)
            .map(|_| V::compute(&mut state.0, &mut state.1))
            .collect();
        SequenceOutput {
            values,
            final_state: state,
        }
    }
    fn checks(_: &RngFixture) -> Vec<Box<dyn Check<SequenceOutput>>> {
        vec![
            Box::new(Exact),
            Box::new(Project {
                name: "byte-entropy",
                project: |output: &SequenceOutput| &output.values,
                check: Box::new(ByteEntropy { minimum: 7.99 }),
            }),
        ]
    }
}
