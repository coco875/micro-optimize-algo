//! Scalar lookup approximations with linear interpolation and std fallbacks.
//! Three shared tables occupy 49,164 bytes of f32 data (48 KiB + 12 bytes).

use std::sync::LazyLock;

const BITS: u32 = 12;
const INTERVALS: usize = 1 << BITS;
const LENGTH: usize = INTERVALS + 1;
const MANTISSA_LOW_BITS: u32 = 23 - BITS;
const MAX_ANGLE: f32 = 1_000_000.0;

struct Tables {
    sine: [f32; LENGTH],
    exp2: [f32; LENGTH],
    log2: [f32; LENGTH],
}

static TABLES: LazyLock<Tables> = LazyLock::new(|| Tables {
    sine: std::array::from_fn(|i| {
        (i as f64 * std::f64::consts::TAU / INTERVALS as f64).sin() as f32
    }),
    exp2: std::array::from_fn(|i| (i as f64 / INTERVALS as f64).exp2() as f32),
    log2: std::array::from_fn(|i| (1.0 + i as f64 / INTERVALS as f64).log2() as f32),
});

/// Initialize once before a benchmark's timer starts, independently of warmups.
pub fn prepare() {
    LazyLock::force(&TABLES);
}

#[inline]
fn interpolate(table: &[f32; LENGTH], index: usize, fraction: f32) -> f32 {
    let left = table[index];
    left + (table[index + 1] - left) * fraction
}

#[inline]
fn sine_at(mut position: f64) -> f32 {
    if position >= INTERVALS as f64 {
        position -= INTERVALS as f64;
    }
    let index = position as usize;
    interpolate(&TABLES.sine, index, (position - index as f64) as f32)
}

#[inline]
fn angle_position(a: f32) -> f64 {
    let cycles = f64::from(a.abs()) / std::f64::consts::TAU;
    // Callers bound |a| to 1,000,000: cycles is positive and fits in u32.
    let whole = cycles as u32;
    (cycles - f64::from(whole)) * INTERVALS as f64
}

#[inline]
pub fn sin(a: f32) -> f32 {
    if !a.is_normal() || a.abs() > MAX_ANGLE {
        return a.sin();
    }
    let sine = sine_at(angle_position(a));
    f32::from_bits(sine.to_bits() ^ (a.to_bits() & (1 << 31)))
}

#[inline]
pub fn cos(a: f32) -> f32 {
    if !a.is_normal() || a.abs() > MAX_ANGLE {
        return a.cos();
    }
    sine_at(angle_position(a) + (INTERVALS / 4) as f64)
}

#[inline]
pub fn tan(a: f32) -> f32 {
    if !a.is_normal() || a.abs() > MAX_ANGLE {
        return a.tan();
    }
    let position = angle_position(a);
    let sine = sine_at(position);
    let cosine = sine_at(position + (INTERVALS / 4) as f64);
    // Avoid amplifying interpolation and rounding errors immediately at poles.
    if cosine.abs() < 0.0001 {
        return a.tan();
    }
    let tangent = sine / cosine;
    f32::from_bits(tangent.to_bits() ^ (a.to_bits() & (1 << 31)))
}

/// Split a positive normal float into its exact integer exponent and an
/// interpolated log2(mantissa), with the mantissa in [1, 2).
#[inline]
fn log2_parts(a: f32) -> (i32, f32) {
    let bits = a.to_bits();
    let exponent = (bits >> 23) as i32 - 127;
    let mantissa = bits & 0x007f_ffff;
    let index = (mantissa >> MANTISSA_LOW_BITS) as usize;
    let remainder = mantissa & ((1 << MANTISSA_LOW_BITS) - 1);
    let fraction = remainder as f32 / (1 << MANTISSA_LOW_BITS) as f32;
    (exponent, interpolate(&TABLES.log2, index, fraction))
}

#[inline]
pub fn log2(a: f32) -> f32 {
    if !a.is_normal() || a < 0.0 {
        return a.log2();
    }
    let (exponent, fraction) = log2_parts(a);
    exponent as f32 + fraction
}

#[inline]
pub fn log(a: f32) -> f32 {
    if !a.is_normal() || a < 0.0 {
        return a.ln();
    }
    let (exponent, fraction) = log2_parts(a);
    ((f64::from(exponent) + f64::from(fraction)) * std::f64::consts::LN_2) as f32
}

/// The caller bounds the exponent to [-126, 126], keeping the scale normal.
#[inline]
fn exp2_at(a: f64) -> f32 {
    let truncated = a as i32;
    let whole = truncated - i32::from(a < f64::from(truncated));
    let position = (a - f64::from(whole)) * INTERVALS as f64;
    // A tiny negative exponent can round its fractional part to exactly 1.
    // Interpolate the last interval at its right endpoint in that case.
    let index = (position as usize).min(INTERVALS - 1);
    let fraction = (position - index as f64) as f32;
    let scale = f32::from_bits(((whole + 127) as u32) << 23);
    scale * interpolate(&TABLES.exp2, index, fraction)
}

#[inline]
pub fn pow2(a: f32) -> f32 {
    if !(-126.0..=126.0).contains(&a) {
        return a.exp2();
    }
    exp2_at(f64::from(a))
}

#[inline]
pub fn pow(a: f32, b: f32) -> f32 {
    if !a.is_normal() || a < 0.0 || !b.is_finite() {
        return a.powf(b);
    }
    let (exponent, fraction) = log2_parts(a);
    let power = (f64::from(exponent) + f64::from(fraction)) * f64::from(b);
    if !(-126.0..=126.0).contains(&power) {
        return a.powf(b);
    }
    exp2_at(power)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::randomness::InputRng;

    #[test]
    fn periodic_tables_cover_negative_angles_and_multiple_periods() {
        prepare();
        for i in 0..=65_536 {
            let angle = -100.0 + 200.0 * i as f32 / 65_536.0;
            assert!(
                (sin(angle) - angle.sin()).abs() <= 0.000_001,
                "sin({angle})"
            );
            assert!(
                (cos(angle) - angle.cos()).abs() <= 0.000_001,
                "cos({angle})"
            );
        }
        for sign in [-1.0, 1.0] {
            for i in 0..=4096 {
                let angle =
                    sign * (std::f32::consts::FRAC_PI_2 - 0.001 - 0.049 * i as f32 / 4096.0);
                let expected = angle.tan();
                let relative = ((tan(angle) - expected) / expected).abs();
                assert!(
                    relative <= 0.000_01,
                    "tan({angle}) relative error={relative}"
                );
            }
        }
    }

    #[test]
    fn logarithm_and_exponential_tables_cover_exponents_and_mantissas() {
        prepare();
        for exponent in 1..255u32 {
            let power_of_two = f32::from_bits(exponent << 23);
            assert_eq!(log2(power_of_two), power_of_two.log2());
            for mantissa in (0..(1 << 23)).step_by(32768).chain([0x007f_ffff]) {
                let input = f32::from_bits((exponent << 23) | mantissa);
                assert!(
                    (log2(input) - input.log2()).abs() <= 0.000_02,
                    "log2({input})"
                );
                assert!((log(input) - input.ln()).abs() <= 0.000_02, "log({input})");
            }
        }
        for i in 0..=65_536 {
            let exponent = -126.0 + 252.0 * i as f32 / 65_536.0;
            let expected = exponent.exp2();
            let relative = ((pow2(exponent) - expected) / expected).abs();
            assert!(
                relative <= 0.000_000_3,
                "pow2({exponent}) relative error={relative}"
            );
        }
    }

    #[test]
    fn powers_use_both_arguments_and_preserve_precision_over_the_workload() {
        prepare();
        let mut rng = InputRng::new(42);
        for _ in 0..65_536 {
            let base = 0.01 + (rng.next_f32_range() + 1.0) * 0.5 * 99.99;
            let exponent = rng.next_f32_range() * 8.0;
            let expected = base.powf(exponent);
            let relative = ((pow(base, exponent) - expected) / expected).abs();
            assert!(
                relative <= 0.000_002,
                "pow({base}, {exponent}) relative error={relative}"
            );
        }
        assert_eq!(pow(2.0, 3.0), 8.0);
        assert_eq!(pow(4.0, -1.0), 0.25);
    }

    #[test]
    fn fallbacks_handle_special_values_large_inputs_and_table_boundaries() {
        type Unary = (fn(f32) -> f32, fn(f32) -> f32);
        let variants: [Unary; 6] = [
            (sin, f32::sin),
            (cos, f32::cos),
            (tan, f32::tan),
            (log, f32::ln),
            (log2, f32::log2),
            (pow2, f32::exp2),
        ];
        for (compute, reference) in variants {
            for input in [
                0.0,
                -0.0,
                f32::from_bits(1),
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::NAN,
            ] {
                let actual = compute(input);
                let expected = reference(input);
                if expected.is_nan() {
                    assert!(actual.is_nan());
                } else {
                    assert_eq!(actual.to_bits(), expected.to_bits(), "input={input}");
                }
            }
        }
        for input in [-f32::MAX, f32::MAX] {
            assert_eq!(sin(input), input.sin());
            assert_eq!(cos(input), input.cos());
            assert_eq!(tan(input), input.tan());
        }
        for input in [std::f32::consts::FRAC_PI_2, -std::f32::consts::FRAC_PI_2] {
            assert_eq!(tan(input), input.tan());
        }
        for input in [-149.0f32, -127.0, 127.0, 128.0] {
            assert_eq!(pow2(input), input.exp2());
        }
        for input in [
            f32::from_bits(0x8000_0001),
            -f32::MIN_POSITIVE,
            -1e-20,
            -1e-12,
        ] {
            assert_eq!(pow2(input), input.exp2(), "tiny negative exponent {input}");
            assert_eq!(pow(2.0, input), 2.0f32.powf(input));
        }
        for (base, exponent) in [(-2.0f32, 3.0f32), (0.0, -2.0), (2.0, 200.0), (0.5, 200.0)] {
            assert_eq!(pow(base, exponent), base.powf(exponent));
        }
        assert!(pow(-2.0, 0.5).is_nan());
    }
}
