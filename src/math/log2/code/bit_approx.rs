use crate::math::log2::Log2Variant;

/// The supplied bit approximation, preserved for comparison.
///
/// This is not a mathematically accurate log2 or an IEEE special-value equivalent
/// of `f32::log2`; see this family's README for examples.
#[inline]
pub fn log2(a: f32) -> f32 {
    let bit = a.to_bits().cast_signed();
    if bit == 0 {
        return f32::NEG_INFINITY;
    }
    if (bit & (1 << 31)) != 0 {
        return f32::NAN;
    }
    let exp = (bit >> 23) - 127;
    let mantissa = (bit & ((1 << 23) - 1)) as u32;
    f32::from_bits((exp as f32).to_bits() + mantissa)
}

#[crate::variant(group = "log2")]
pub struct BitApprox;

impl Log2Variant for BitApprox {
    #[inline]
    fn compute(a: f32) -> f32 {
        log2(a)
    }
}
