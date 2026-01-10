//! x86_64 AVX2 SIMD implementation.
//!
//! This implementation uses AVX2 intrinsics to process 8 f32 values
//! simultaneously, providing significant speedup on compatible CPUs.

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// Compute the dot product using AVX2 SIMD instructions.
///
/// Processes 8 f32 values per iteration using 256-bit registers.
///
/// # Safety
/// This function requires AVX2 support. It is conditionally compiled
/// only when the target supports AVX2.
///
/// # Arguments
/// * `a` - First vector
/// * `b` - Second vector
///
/// # Panics
/// Panics if the vectors have different lengths.
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub fn dot_product_x86_64_avx2(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");

    let len = a.len();

    if len < 8 {
        // Fall back to scalar for small vectors
        return a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    }

    unsafe {
        let chunks = len / 8;
        let remainder = len % 8;

        // Initialize accumulator to zero
        let mut sum_vec = _mm256_setzero_ps();

        for i in 0..chunks {
            let idx = i * 8;
            // Load 8 floats from each vector
            let a_vec = _mm256_loadu_ps(a.as_ptr().add(idx));
            let b_vec = _mm256_loadu_ps(b.as_ptr().add(idx));

            // Multiply and accumulate using FMA if available
            #[cfg(target_feature = "fma")]
            {
                sum_vec = _mm256_fmadd_ps(a_vec, b_vec, sum_vec);
            }
            #[cfg(not(target_feature = "fma"))]
            {
                let prod = _mm256_mul_ps(a_vec, b_vec);
                sum_vec = _mm256_add_ps(sum_vec, prod);
            }
        }

        // Horizontal sum of the 256-bit register
        // sum_vec = [a, b, c, d, e, f, g, h]
        let hi = _mm256_extractf128_ps(sum_vec, 1); // [e, f, g, h]
        let lo = _mm256_castps256_ps128(sum_vec); // [a, b, c, d]
        let sum128 = _mm_add_ps(lo, hi); // [a+e, b+f, c+g, d+h]

        let shuf = _mm_movehdup_ps(sum128); // [b+f, b+f, d+h, d+h]
        let sums = _mm_add_ps(sum128, shuf); // [a+e+b+f, ...]
        let shuf2 = _mm_movehl_ps(sums, sums); // [..., c+g+d+h, ...]
        let sums2 = _mm_add_ss(sums, shuf2); // [sum of all 8]

        let mut result = _mm_cvtss_f32(sums2);

        // Handle remaining elements
        let base = chunks * 8;
        for i in 0..remainder {
            result += a[base + i] * b[base + i];
        }

        result
    }
}

/// Fallback for non-AVX2 builds (should not be called)
#[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
pub fn dot_product_x86_64_avx2(a: &[f32], b: &[f32]) -> f32 {
    // This should never be called on non-AVX2 systems
    // Fall back to basic implementation
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}
