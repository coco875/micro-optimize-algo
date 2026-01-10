//! x86_64 SSE2 SIMD implementation.
//!
//! SSE2 is available on all x86_64 CPUs, processing 4 f32 values per iteration.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Compute the dot product using SSE2 SIMD instructions.
///
/// Processes 4 f32 values per iteration using 128-bit registers.
/// Available on all x86_64 CPUs.
#[cfg(target_arch = "x86_64")]
pub fn dot_product_x86_64_sse2(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");

    let len = a.len();

    if len < 4 {
        return a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    }

    unsafe {
        let chunks = len / 4;
        let remainder = len % 4;

        let mut sum_vec = _mm_setzero_ps();

        for i in 0..chunks {
            let idx = i * 4;
            let a_vec = _mm_loadu_ps(a.as_ptr().add(idx));
            let b_vec = _mm_loadu_ps(b.as_ptr().add(idx));
            let prod = _mm_mul_ps(a_vec, b_vec);
            sum_vec = _mm_add_ps(sum_vec, prod);
        }

        // Horizontal sum of 128-bit register
        // sum_vec = [a, b, c, d]
        let shuf = _mm_movehdup_ps(sum_vec); // [b, b, d, d]
        let sums = _mm_add_ps(sum_vec, shuf); // [a+b, _, c+d, _]
        let shuf2 = _mm_movehl_ps(sums, sums); // [c+d, _, _, _]
        let sums2 = _mm_add_ss(sums, shuf2); // [a+b+c+d, _, _, _]

        let mut result = _mm_cvtss_f32(sums2);

        // Handle remainder
        let base = chunks * 4;
        for i in 0..remainder {
            result += a[base + i] * b[base + i];
        }

        result
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn dot_product_x86_64_sse2(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}
