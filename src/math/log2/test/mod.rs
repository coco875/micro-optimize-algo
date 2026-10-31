use super::code::std::Std;
use super::scenarios::random::StdErrorReport;
use super::{log2, Log2Variant};
use crate::utils::validation::Check;

#[test]
fn alternative_approximations_stay_within_sampled_error_budgets() {
    use super::code::{atanh_series::AtanhSeries, linear::Linear, quadratic::Quadratic};
    type Approximation = (&'static str, fn(f32) -> f32, f64);
    let variants: [Approximation; 3] = [
        ("Linear", Linear::compute, 0.087),
        ("Quadratic", Quadratic::compute, 0.011),
        ("AtanhSeries", AtanhSeries::compute, 0.000_02),
    ];
    for (name, compute, limit) in variants {
        for exponent in 1..255u32 {
            let input = f32::from_bits(exponent << 23);
            assert_eq!(compute(input), input.log2(), "{name}, input={input}");
        }
        for exponent in [1u32, 62, 126, 127, 128, 190, 253, 254] {
            for mantissa in (0..(1 << 23)).step_by(512).chain([0x007f_ffff]) {
                let input = f32::from_bits((exponent << 23) | mantissa);
                let error = (f64::from(compute(input)) - f64::from(input.log2())).abs();
                assert!(
                    error <= limit,
                    "{name}, input={input}, error={error}, limit={limit}"
                );
                if exponent == 127 && name == "AtanhSeries" {
                    assert!(error <= 0.000_002, "{name}, input={input}, error={error}");
                }
            }
        }
        for input in [
            0.0,
            -0.0,
            -1.0,
            -f32::MIN_POSITIVE,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
            f32::from_bits(1),
            f32::from_bits(0x007f_ffff),
            f32::from_bits(0x8000_0001),
        ] {
            let expected = input.log2();
            let actual = compute(input);
            if expected.is_nan() {
                assert!(actual.is_nan(), "{name}, input={input}");
            } else {
                assert_eq!(
                    actual.to_bits(),
                    expected.to_bits(),
                    "{name}, input={input}"
                );
            }
        }
    }
}

#[test]
fn normal_powers_of_two_match_std() {
    for exponent in 1..255u32 {
        let input = f32::from_bits(exponent << 23);
        assert_eq!(log2(input), Std::compute(input), "input={input}");
    }
}

#[test]
fn supplied_approximation_differs_for_fractional_mantissas() {
    assert_eq!(log2(1.5).to_bits(), 1 << 22);
    assert!((Std::compute(1.5) - 0.584_962_5).abs() < f32::EPSILON);
    assert!(log2(1.5) < f32::MIN_POSITIVE);
    assert_eq!(log2(0.75), -1.5);
    assert!((Std::compute(0.75) + 0.415_037_5).abs() < f32::EPSILON);
}

#[test]
fn special_values_preserve_supplied_behavior() {
    assert_eq!(log2(0.0), f32::NEG_INFINITY);
    assert!(log2(-1.0).is_nan());
    assert!(log2(f32::NEG_INFINITY).is_nan());
    assert!(log2(-0.0).is_nan());
    assert_eq!(Std::compute(-0.0), f32::NEG_INFINITY);
    assert_eq!(log2(f32::INFINITY), 128.0);
    assert_eq!(Std::compute(f32::INFINITY), f32::INFINITY);
    assert!(log2(f32::NAN).is_finite());
    assert!(Std::compute(f32::NAN).is_nan());
    let subnormal = f32::from_bits(1);
    assert_eq!(Std::compute(subnormal), -149.0);
    assert_ne!(log2(subnormal), Std::compute(subnormal));
}

#[test]
fn accuracy_report_exposes_error_and_rejects_invalid_observations() {
    let reference = vec![0.0, 1.0];
    let result = StdErrorReport.evaluate(&vec![0.0, 3.0], Some(&reference));
    assert!(result.passed);
    assert!(result.measured.contains("max_abs=2.00000000"));
    assert!(result.measured.contains("mismatches=1/2"));
    assert!(!StdErrorReport.evaluate(&vec![], Some(&vec![])).passed);
    assert!(!StdErrorReport.evaluate(&vec![0.0], Some(&reference)).passed);
    assert!(!StdErrorReport.evaluate(&reference, None).passed);
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(
            !StdErrorReport
                .evaluate(&vec![invalid, 1.0], Some(&reference))
                .passed
        );
    }
}
