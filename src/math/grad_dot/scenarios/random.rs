use crate::math::grad_dot::{code::GRADIENT, GradDotVariant};
use crate::utils::randomness::InputRng;
use crate::utils::scenario::{size_params, Scenario, ScenarioDefinition, ValidationCase};
use crate::utils::validation::{Check, NumericTolerance};

#[derive(Clone)]
pub struct GradFixture {
    hashes: Vec<usize>,
    x: Vec<f64>,
    y: Vec<f64>,
    z: Vec<f64>,
}
#[crate::scenario(
    group = "grad_dot",
    id = "random",
    name = "GradDot",
    tags = "math, noise, 3d noise"
)]
pub struct GradDotScenario<V> {
    fixture: GradFixture,
    marker: std::marker::PhantomData<V>,
}
impl<V: GradDotVariant> Scenario for GradDotScenario<V> {
    fn task(&mut self) {
        let f = std::hint::black_box(&self.fixture);
        std::hint::black_box(V::compute(&f.hashes, &f.x, &f.y, &f.z));
    }
}
impl<V: GradDotVariant> ScenarioDefinition<V> for GradDotScenario<V> {
    type Param = usize;
    type Fixture = GradFixture;
    type Output = f64;
    fn prepare(size: &usize, rng: &mut InputRng) -> GradFixture {
        GradFixture {
            hashes: (0..*size)
                .map(|_| rng.next_u32_range(256) as usize)
                .collect(),
            x: (0..*size)
                .map(|_| f64::from(rng.next_f32_range()))
                .collect(),
            y: (0..*size)
                .map(|_| f64::from(rng.next_f32_range()))
                .collect(),
            z: (0..*size)
                .map(|_| f64::from(rng.next_f32_range()))
                .collect(),
        }
    }
    fn new(fixture: &GradFixture) -> Self {
        Self {
            fixture: fixture.clone(),
            marker: std::marker::PhantomData,
        }
    }
    fn generate_params(config: &crate::utils::runner::Config) -> Vec<usize> {
        size_params(config, "grad_dot")
    }
    fn format_param(param: &usize) -> String {
        param.to_string()
    }
    fn validation_cases(_: &crate::utils::runner::Config) -> Vec<ValidationCase<usize>> {
        [0, 1, 3, 4, 7, 17, 64, 1024]
            .into_iter()
            .map(|n| ValidationCase::new(n.to_string(), n))
            .collect()
    }
    fn validate(f: &GradFixture) -> f64 {
        V::compute(&f.hashes, &f.x, &f.y, &f.z)
    }
    fn checks(f: &GradFixture) -> Vec<Box<dyn Check<f64>>> {
        let terms: Vec<_> = f
            .hashes
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let g = GRADIENT[h & 15];
                g[0] * f.x[i] + g[1] * f.y[i] + g[2] * f.z[i]
            })
            .collect();
        let absolute = terms.iter().map(|v| v.abs()).sum::<f64>()
            * (terms.len() as f64 + 1.0)
            * f64::EPSILON
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
                expected: Some(terms.iter().sum()),
            }),
        ]
    }
}
