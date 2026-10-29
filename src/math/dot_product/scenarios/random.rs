use crate::math::dot_product::DotProductVariant;
use crate::utils::randomness::InputRng;
use crate::utils::scenario::{size_params, Scenario, ScenarioDefinition, ValidationCase};
use crate::utils::validation::{Check, NumericTolerance};

pub struct DotFixture {
    pub a: Vec<f32>,
    pub b: Vec<f32>,
}

#[crate::scenario(
    group = "dot_product",
    id = "random",
    name = "DotProduct",
    tags = "math, vector, 1d"
)]
pub struct DotProductScenario<V> {
    fixture: DotFixture,
    marker: std::marker::PhantomData<V>,
}

pub fn oracle(fixture: &DotFixture) -> f64 {
    fixture
        .a
        .iter()
        .zip(&fixture.b)
        .map(|(&a, &b)| f64::from(a) * f64::from(b))
        .sum()
}

/// Bound f32 rounding against f64, scaled by the sum of absolute products.
/// Four n*epsilon budgets cover multiplication and different accumulation trees.
pub fn checks(fixture: &DotFixture) -> Vec<Box<dyn Check<f64>>> {
    let absolute = fixture
        .a
        .iter()
        .zip(&fixture.b)
        .map(|(&a, &b)| (f64::from(a) * f64::from(b)).abs())
        .sum::<f64>()
        * (fixture.a.len() as f64 + 1.0)
        * f64::from(f32::EPSILON)
        * 4.0;
    vec![
        Box::new(NumericTolerance {
            absolute,
            relative: 0.0,
            expected: None,
        }),
        Box::new(NumericTolerance {
            absolute,
            relative: 0.0,
            expected: Some(oracle(fixture)),
        }),
    ]
}

impl<V: DotProductVariant> Scenario for DotProductScenario<V> {
    fn task(&mut self) {
        std::hint::black_box(V::compute(
            std::hint::black_box(&self.fixture.a),
            std::hint::black_box(&self.fixture.b),
        ));
    }
}
impl<V: DotProductVariant> ScenarioDefinition<V> for DotProductScenario<V> {
    type Param = usize;
    type Fixture = DotFixture;
    type Output = f64;
    fn prepare(size: &usize, rng: &mut InputRng) -> DotFixture {
        DotFixture {
            a: (0..*size).map(|_| rng.next_f32_range()).collect(),
            b: (0..*size).map(|_| rng.next_f32_range()).collect(),
        }
    }
    fn new(fixture: &DotFixture) -> Self {
        Self {
            fixture: DotFixture {
                a: fixture.a.clone(),
                b: fixture.b.clone(),
            },
            marker: std::marker::PhantomData,
        }
    }
    fn generate_params(config: &crate::utils::runner::Config) -> Vec<usize> {
        size_params(config, "dot_product")
    }
    fn format_param(param: &usize) -> String {
        param.to_string()
    }
    fn validation_cases(_: &crate::utils::runner::Config) -> Vec<ValidationCase<usize>> {
        [0, 1, 3, 4, 7, 8, 17, 64, 1024]
            .into_iter()
            .map(|n| ValidationCase::new(n.to_string(), n))
            .collect()
    }
    fn validate(fixture: &DotFixture) -> f64 {
        f64::from(V::compute(&fixture.a, &fixture.b))
    }
    fn checks(fixture: &DotFixture) -> Vec<Box<dyn Check<f64>>> {
        checks(fixture)
    }
}
