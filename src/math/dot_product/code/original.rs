//! Original (reference) implementation of dot product.
//!
//! This is a clean, idiomatic Rust implementation that serves as the
//! baseline for correctness and performance comparison.

/// Compute the dot product of two vectors.
///
/// # Arguments
/// * `a` - First vector
/// * `b` - Second vector
///
/// # Panics
/// Panics if the vectors have different lengths.
///
/// # Example
/// ```
/// use micro_optimize_algo::math::dot_product::dot_product_original;
///
/// let a = [1.0, 2.0, 3.0];
/// let b = [4.0, 5.0, 6.0];
/// let result = dot_product_original(&a, &b);
/// assert!((result - 32.0).abs() < 1e-6);
/// ```
pub fn dot_product_original(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");

    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}
