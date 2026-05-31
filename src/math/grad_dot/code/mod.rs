//! Implementations of the gradient dot product algorithm.

use std::simd::prelude::*;
use crate::utils::VariantInfo;

/// Gradient vectors shared between Perlin and simplex noise (from vanilla `SimplexNoise.GRADIENT`).
pub const GRADIENT: [[f64; 3]; 16] = [
    [1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0],
    [1.0, -1.0, 0.0],
    [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [1.0, 0.0, -1.0],
    [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0],
    [0.0, -1.0, 1.0],
    [0.0, 1.0, -1.0],
    [0.0, -1.0, -1.0],
    [1.0, 1.0, 0.0],
    [0.0, -1.0, 1.0],
    [-1.0, 1.0, 0.0],
    [0.0, -1.0, -1.0],
];

/// 4-component gradient vectors for vectorization.
pub const GRADIENT_4: [[f64; 4]; 16] = [
    [1.0, 1.0, 0.0, 0.0],
    [-1.0, 1.0, 0.0, 0.0],
    [1.0, -1.0, 0.0, 0.0],
    [-1.0, -1.0, 0.0, 0.0],
    [1.0, 0.0, 1.0, 0.0],
    [-1.0, 0.0, 1.0, 0.0],
    [1.0, 0.0, -1.0, 0.0],
    [-1.0, 0.0, -1.0, 0.0],
    [0.0, 1.0, 1.0, 0.0],
    [0.0, -1.0, 1.0, 0.0],
    [0.0, 1.0, -1.0, 0.0],
    [0.0, -1.0, -1.0, 0.0],
    [1.0, 1.0, 0.0, 0.0],
    [0.0, -1.0, 1.0, 0.0],
    [-1.0, 1.0, 0.0, 0.0],
    [0.0, -1.0, -1.0, 0.0],
];

/// Calculate the dot product of a gradient vector and the position vector.
#[expect(clippy::inline_always, reason = "hot-path noise primitive")]
#[inline(always)]
#[must_use]
pub fn grad_dot(hash: usize, x: f64, y: f64, z: f64) -> f64 {
    let g = &GRADIENT[hash & 15];
    g[0] * x + g[1] * y + g[2] * z
}

/// Gather gradient components for 4 hashes into separate x/y/z SIMD vectors,
/// then compute the dot product with the given position vectors using loop.
#[inline]
#[must_use]
pub fn grad_dot_4x_loop(hashes: [usize; 4], x: f64x4, y: f64x4, z: f64x4) -> f64x4 {
    let mut gx = [0.0f64; 4];
    let mut gy = [0.0f64; 4];
    let mut gz = [0.0f64; 4];
    for i in 0..4 {
        let g = &GRADIENT[hashes[i] & 15];
        gx[i] = g[0];
        gy[i] = g[1];
        gz[i] = g[2];
    }
    f64x4::from_array(gx) * x + f64x4::from_array(gy) * y + f64x4::from_array(gz) * z
}

/// Gather gradient components for 4 hashes into separate x/y/z SIMD vectors,
/// then compute the dot product with the given position vectors.
#[inline]
#[must_use]
pub fn grad_dot_4x(hashes: [usize; 4], x: f64x4, y: f64x4, z: f64x4) -> f64x4 {
    let h0 = f64x4::from_array(GRADIENT_4[hashes[0] & 15]);
    let h1 = f64x4::from_array(GRADIENT_4[hashes[1] & 15]);
    let h2 = f64x4::from_array(GRADIENT_4[hashes[2] & 15]);
    let h3 = f64x4::from_array(GRADIENT_4[hashes[3] & 15]);

    let (gx, gy, gz, _gw) = transpose(h0, h1, h2, h3);

    gx * x + gy * y + gz * z
}

/// Transpose a 4x4 matrix of f64s using deinterleave.
#[inline]
#[must_use]
pub fn transpose(
    r0: f64x4, // [a0, a1, a2, a3]
    r1: f64x4, // [b0, b1, b2, b3]
    r2: f64x4, // [c0, c1, c2, c3]
    r3: f64x4, // [d0, d1, d2, d3]
) -> (f64x4, f64x4, f64x4, f64x4) {
    let (t0, t1) = r0.deinterleave(r1); // t0 = [a0, a2, b0, b2], t1 = [a1, a3, b1, b3]
    let (t2, t3) = r2.deinterleave(r3); // t2 = [c0, c2, d0, d2], t3 = [c1, c3, d1, d3]

    let (col0, col2) = t0.deinterleave(t2); // col0 = [a0, b0, c0, d0], col2 = [a2, b2, c2, d2]
    let (col1, col3) = t1.deinterleave(t3); // col1 = [a1, b1, c1, d1], col3 = [a3, b3, c3, d3]

    (col0, col1, col2, col3)
}

/// Original (reference) implementation of slice-wide gradient dot product.
pub fn grad_dot_original(hashes: &[usize], x: &[f64], y: &[f64], z: &[f64]) -> f64 {
    assert_eq!(hashes.len(), x.len(), "Vectors must have the same length");
    assert_eq!(hashes.len(), y.len(), "Vectors must have the same length");
    assert_eq!(hashes.len(), z.len(), "Vectors must have the same length");

    hashes
        .iter()
        .zip(x.iter())
        .zip(y.iter())
        .zip(z.iter())
        .map(|(((h, &x_val), &y_val), &z_val)| {
            grad_dot(*h, x_val, y_val, z_val)
        })
        .sum()
}

/// Vectorized implementation using `portable_simd` (f64x4) with manual loop gather.
pub fn grad_dot_simd_portable_x4(hashes: &[usize], x: &[f64], y: &[f64], z: &[f64]) -> f64 {
    assert_eq!(hashes.len(), x.len(), "Vectors must have the same length");
    assert_eq!(hashes.len(), y.len(), "Vectors must have the same length");
    assert_eq!(hashes.len(), z.len(), "Vectors must have the same length");

    let chunks = hashes.len() / 4;
    let mut sum_v = f64x4::splat(0.0);

    for c in 0..chunks {
        let idx = c * 4;
        let h = [hashes[idx], hashes[idx + 1], hashes[idx + 2], hashes[idx + 3]];
        let vx = f64x4::from_slice(&x[idx..idx + 4]);
        let vy = f64x4::from_slice(&y[idx..idx + 4]);
        let vz = f64x4::from_slice(&z[idx..idx + 4]);
        sum_v += grad_dot_4x_loop(h, vx, vy, vz);
    }

    let mut total = sum_v.reduce_sum();

    // Handle remainder elements
    for i in (chunks * 4)..hashes.len() {
        total += grad_dot(hashes[i], x[i], y[i], z[i]);
    }

    total
}

/// Vectorized implementation using `portable_simd` (f64x4) with transpose gather.
pub fn grad_dot_simd_transpose_x4(hashes: &[usize], x: &[f64], y: &[f64], z: &[f64]) -> f64 {
    assert_eq!(hashes.len(), x.len(), "Vectors must have the same length");
    assert_eq!(hashes.len(), y.len(), "Vectors must have the same length");
    assert_eq!(hashes.len(), z.len(), "Vectors must have the same length");

    let chunks = hashes.len() / 4;
    let mut sum_v = f64x4::splat(0.0);

    for c in 0..chunks {
        let idx = c * 4;
        let h = [hashes[idx], hashes[idx + 1], hashes[idx + 2], hashes[idx + 3]];
        let vx = f64x4::from_slice(&x[idx..idx + 4]);
        let vy = f64x4::from_slice(&y[idx..idx + 4]);
        let vz = f64x4::from_slice(&z[idx..idx + 4]);
        sum_v += grad_dot_4x(h, vx, vy, vz);
    }

    let mut total = sum_v.reduce_sum();

    // Handle remainder elements
    for i in (chunks * 4)..hashes.len() {
        total += grad_dot(hashes[i], x[i], y[i], z[i]);
    }

    total
}

/// Type alias for gradient dot product function signature
pub type GradDotFn = fn(&[usize], &[f64], &[f64], &[f64]) -> f64;

/// Get all available variants
pub fn available_variants() -> Vec<VariantInfo<GradDotFn>> {
    vec![
        VariantInfo {
            name: "original",
            description: "Scalar gradient dot product reference implementation",
            function: grad_dot_original,
        },
        VariantInfo {
            name: "simd_portable_x4",
            description: "Vectorized implementation using portable_simd f64x4 (loop gather)",
            function: grad_dot_simd_portable_x4,
        },
        VariantInfo {
            name: "simd_transpose_x4",
            description: "Vectorized implementation using portable_simd f64x4 (transpose gather)",
            function: grad_dot_simd_transpose_x4,
        },
    ]
}
