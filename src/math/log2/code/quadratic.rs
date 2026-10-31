use super::decompose;
use crate::math::log2::Log2Variant;

/// Interpolate log2(m) at m = 1, 1.5, and 2 with a quadratic in t = m - 1.
#[inline]
pub fn log2_quadratic(a: f32) -> f32 {
    let Some((exponent, mantissa)) = decompose(a) else {
        return a.log2();
    };
    let t = mantissa - 1.0;
    // A = 4*log2(1.5) - 1, B = 1 - A, rounded to f32.
    exponent + t * (1.339_85 - 0.339_85 * t)
}

#[crate::variant(group = "log2")]
pub struct Quadratic;

impl Log2Variant for Quadratic {
    #[inline]
    fn compute(a: f32) -> f32 {
        log2_quadratic(a)
    }
}
