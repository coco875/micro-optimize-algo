//! Optimized scalar implementation with loop unrolling.
//!
//! This implementation uses manual loop unrolling to reduce loop overhead
//! and allow the CPU to better utilize instruction-level parallelism.

/// Compute the dot product with 4x loop unrolling.
///
/// This implementation processes 4 elements per iteration, reducing
/// loop overhead and enabling better instruction pipelining.
///
/// # Arguments
/// * `a` - First vector
/// * `b` - Second vector
///
/// # Panics
/// Panics if the vectors have different lengths.
pub fn dot_product_scalar_opt(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");

    let len = a.len();
    let chunks = len / 4;
    let remainder = len % 4;

    // Process 4 elements at a time with 4 accumulators
    // to reduce data dependencies
    let mut sum0: f32 = 0.0;
    let mut sum1: f32 = 0.0;
    let mut sum2: f32 = 0.0;
    let mut sum3: f32 = 0.0;

    for i in 0..chunks {
        let idx = i * 4;
        sum0 += a[idx] * b[idx];
        sum1 += a[idx + 1] * b[idx + 1];
        sum2 += a[idx + 2] * b[idx + 2];
        sum3 += a[idx + 3] * b[idx + 3];
    }

    // Handle remaining elements
    let base = chunks * 4;
    for i in 0..remainder {
        sum0 += a[base + i] * b[base + i];
    }

    // Combine all partial sums
    (sum0 + sum1) + (sum2 + sum3)
}
