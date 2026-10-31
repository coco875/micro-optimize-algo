use super::{
    code::variants::{Libm, Std},
    Pow2Variant,
};

#[test]
fn known_answers_for_std_and_libm() {
    let variants: [fn(f32) -> f32; 2] = [Std::compute, Libm::compute];
    for compute in variants {
        for (a, expected) in [(-1.0, 0.5), (0.0, 1.0), (1.0, 2.0), (3.0, 8.0)] {
            let actual = compute(a);
            assert!(
                (actual - expected).abs() <= 1e-6,
                "input={a}, actual={actual}, expected={expected}"
            );
        }
    }
}
