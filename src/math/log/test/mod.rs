use super::{
    code::variants::{Libm, Std},
    LogVariant,
};

#[test]
fn known_answers_for_std_and_libm() {
    let variants: [fn(f32) -> f32; 2] = [Std::compute, Libm::compute];
    for compute in variants {
        for (a, expected) in [
            (1.0, 0.0),
            (std::f32::consts::E, 1.0),
            (2.0, std::f32::consts::LN_2),
        ] {
            let actual = compute(a);
            assert!(
                (actual - expected).abs() <= 1e-6,
                "input={a}, actual={actual}, expected={expected}"
            );
        }
    }
}
