use super::{
    code::variants::{Libm, Std},
    CosVariant,
};

#[test]
fn known_answers_for_std_and_libm() {
    let variants: [fn(f32) -> f32; 2] = [Std::compute, Libm::compute];
    for compute in variants {
        for (a, expected) in [(0.0, 1.0), (std::f32::consts::PI, -1.0)] {
            let actual = compute(a);
            assert!(
                (actual - expected).abs() <= 1e-6,
                "input={a}, actual={actual}, expected={expected}"
            );
        }
    }
}
