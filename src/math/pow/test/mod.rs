use super::{
    code::variants::{Libm, Std},
    PowVariant,
};

#[test]
fn known_answers_for_std_and_libm() {
    let variants: [fn(f32, f32) -> f32; 2] = [Std::compute, Libm::compute];
    for compute in variants {
        for (a, b, expected) in [
            (0.25, 0.5, 0.5),
            (2.0, 3.0, 8.0),
            (4.0, -1.0, 0.25),
            (1.0, 7.0, 1.0),
        ] {
            let actual = compute(a, b);
            assert!(
                (actual - expected).abs() <= 1e-6,
                "input={a}, actual={actual}, expected={expected}"
            );
        }
    }
}
