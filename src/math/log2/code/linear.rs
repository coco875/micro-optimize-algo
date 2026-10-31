use super::decompose;
use crate::math::log2::Log2Variant;

/// Linear interpolation between the logarithms of adjacent powers of two.
#[inline]
pub fn log2_linear(a: f32) -> f32 {
    let Some((exponent, mantissa)) = decompose(a) else {
        return a.log2();
    };
    exponent + (mantissa - 1.0)
}

#[crate::variant(group = "log2")]
pub struct Linear;

impl Log2Variant for Linear {
    #[inline]
    fn compute(a: f32) -> f32 {
        log2_linear(a)
    }
}
