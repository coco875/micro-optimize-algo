use crate::math::log2::Log2Variant;
use crate::utils::randomness::InputRng;
use crate::utils::runner::Config;
use crate::utils::scenario::{size_params, Scenario, ScenarioDefinition, ValidationCase};
use crate::utils::validation::{Check, CheckResult};

#[derive(Clone, Copy)]
pub enum Domain {
    PositiveNormal,
    OneToTwo,
}

impl Domain {
    fn label(self) -> &'static str {
        match self {
            Self::PositiveNormal => "positive-normal",
            Self::OneToTwo => "one-to-two",
        }
    }
}

#[crate::scenario(
    group = "log2",
    id = "random",
    name = "Log2",
    tags = "math, logarithm, approximation"
)]
pub struct Log2Scenario<V> {
    inputs: Vec<f32>,
    marker: std::marker::PhantomData<V>,
}

impl<V: Log2Variant> Scenario for Log2Scenario<V> {
    fn task(&mut self) {
        for &input in std::hint::black_box(&self.inputs) {
            std::hint::black_box(V::compute(std::hint::black_box(input)));
        }
    }
}

impl<V: Log2Variant> ScenarioDefinition<V> for Log2Scenario<V> {
    type Param = (Domain, usize);
    type Fixture = Vec<f32>;
    type Output = Vec<f32>;

    fn prepare(&(domain, size): &Self::Param, rng: &mut InputRng) -> Vec<f32> {
        (0..size)
            .map(|_| {
                let exponent = match domain {
                    Domain::PositiveNormal => 1 + rng.next_u32_range(254),
                    Domain::OneToTwo => 127,
                };
                f32::from_bits((exponent << 23) | rng.next_u32_range(1 << 23))
            })
            .collect()
    }

    fn new(inputs: &Vec<f32>) -> Self {
        V::prepare();
        Self {
            inputs: inputs.clone(),
            marker: std::marker::PhantomData,
        }
    }

    fn generate_params(config: &Config) -> Vec<Self::Param> {
        [Domain::PositiveNormal, Domain::OneToTwo]
            .into_iter()
            .flat_map(|domain| {
                size_params(config, "log2")
                    .into_iter()
                    .map(move |n| (domain, n))
            })
            .collect()
    }

    fn format_param(&(domain, size): &Self::Param) -> String {
        format!("{}/n={size}", domain.label())
    }

    fn validation_cases(_: &Config) -> Vec<ValidationCase<Self::Param>> {
        [Domain::PositiveNormal, Domain::OneToTwo]
            .into_iter()
            .map(|domain| ValidationCase::new(domain.label(), (domain, 65_536)))
            .collect()
    }

    fn validate(inputs: &Vec<f32>) -> Vec<f32> {
        V::prepare();
        inputs.iter().copied().map(V::compute).collect()
    }

    fn checks(_: &Vec<f32>) -> Vec<Box<dyn Check<Vec<f32>>>> {
        vec![Box::new(StdErrorReport)]
    }
}

/// Characterize accuracy without claiming that the supplied approximation is correct.
pub struct StdErrorReport;

impl Check<Vec<f32>> for StdErrorReport {
    fn name(&self) -> &str {
        "std-error-report"
    }

    fn evaluate(&self, actual: &Vec<f32>, reference: Option<&Vec<f32>>) -> CheckResult {
        let Some(reference) = reference else {
            return CheckResult::new(
                false,
                "missing reference",
                "std required",
                "accuracy report",
            );
        };
        if actual.is_empty() || actual.len() != reference.len() {
            return CheckResult::new(
                false,
                format!("{} outputs", actual.len()),
                format!("{} nonempty outputs", reference.len()),
                "accuracy report requires matching observations",
            );
        }
        let mut max_absolute = 0.0f64;
        let mut squared_error = 0.0;
        let mut mismatches = 0;
        let mut finite = true;
        for (&actual, &expected) in actual.iter().zip(reference) {
            finite &= actual.is_finite() && expected.is_finite();
            let error = (f64::from(actual) - f64::from(expected)).abs();
            max_absolute = max_absolute.max(error);
            squared_error += error * error;
            mismatches += usize::from(actual != expected);
        }
        let rmse = (squared_error / actual.len() as f64).sqrt();
        CheckResult::new(
            finite,
            format!(
                "max_abs={max_absolute:.8}, rmse={rmse:.8}, mismatches={mismatches}/{}",
                actual.len()
            ),
            "finite outputs; accuracy metrics are informational",
            "comparison with f32::log2, not a correctness tolerance",
        )
        .with_metric("max_abs", max_absolute)
        .with_metric("rmse", rmse)
        .with_metric("mismatches", mismatches)
        .with_metric("samples", actual.len())
    }
}
