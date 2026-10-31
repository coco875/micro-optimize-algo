use super::decompose;
use crate::math::log2::Log2Variant;

/// Five odd-power terms of log2(m) = 2*log2(e)*atanh((m-1)/(m+1)).
#[inline]
pub fn log2_atanh_series(a: f32) -> f32 {
    let Some((exponent, mantissa)) = decompose(a) else {
        return a.log2();
    };
    let z = (mantissa - 1.0) / (mantissa + 1.0);
    let z2 = z * z;
    let polynomial = 1.0 + z2 * (1.0 / 3.0 + z2 * (1.0 / 5.0 + z2 * (1.0 / 7.0 + z2 / 9.0)));
    exponent + (2.0 * std::f32::consts::LOG2_E) * z * polynomial
}

#[crate::variant(group = "log2")]
pub struct AtanhSeries;

impl Log2Variant for AtanhSeries {
    #[inline]
    fn compute(a: f32) -> f32 {
        log2_atanh_series(a)
    }
}
