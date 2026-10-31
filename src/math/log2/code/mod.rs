//! Shared range reduction for the alternative approximations.

/// For a positive normal input, return its exponent and a mantissa in [1, 2).
/// Other inputs use std in each variant to preserve its special-value behavior.
#[inline]
fn decompose(a: f32) -> Option<(f32, f32)> {
    let bits = a.to_bits();
    if !(0x0080_0000..0x7f80_0000).contains(&bits) {
        return None;
    }
    let exponent = ((bits >> 23) as i32 - 127) as f32;
    let mantissa = f32::from_bits(0x3f80_0000 | (bits & 0x007f_ffff));
    Some((exponent, mantissa))
}

crate::implementation_modules!("math/log2");
