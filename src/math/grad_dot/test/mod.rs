//! Tests for gradient dot product implementations.

#[cfg(test)]
mod tests {
    use crate::math::grad_dot::code::*;

    const EPSILON: f64 = 1e-12;

    fn assert_close(a: f64, b: f64, msg: &str) {
        let diff = (a - b).abs();
        assert!(
            diff < EPSILON,
            "{}: expected {}, got {}, diff = {}",
            msg,
            b,
            a,
            diff
        );
    }

    #[test]
    fn test_scalar_vs_simd() {
        let hashes = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
        let x = vec![
            0.5, -0.5, 0.25, -0.25, 0.1, -0.1, 0.2, -0.2, 0.3, -0.3, 0.4, -0.4, 0.5, -0.5, 0.6,
            -0.6, 0.7, -0.7,
        ];
        let y = vec![
            0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, -1.0, -0.9, -0.8, -0.7, -0.6, -0.5,
            -0.4, -0.3,
        ];
        let z = vec![
            -0.5, 0.5, -0.25, 0.25, -0.1, 0.1, -0.2, 0.2, -0.3, 0.3, -0.4, 0.4, -0.5, 0.5, -0.6,
            0.6, -0.7, 0.7,
        ];

        let res_scalar = grad_dot_original(&hashes, &x, &y, &z);
        let res_simd = grad_dot_simd_portable_x4(&hashes, &x, &y, &z);
        let res_transpose = grad_dot_simd_transpose_x4(&hashes, &x, &y, &z);

        assert_close(res_simd, res_scalar, "Scalar vs Loop SIMD");
        assert_close(res_transpose, res_scalar, "Scalar vs Transpose SIMD");
    }

    #[test]
    fn test_empty() {
        let hashes: Vec<usize> = vec![];
        let x: Vec<f64> = vec![];
        let y: Vec<f64> = vec![];
        let z: Vec<f64> = vec![];

        let res_scalar = grad_dot_original(&hashes, &x, &y, &z);
        let res_simd = grad_dot_simd_portable_x4(&hashes, &x, &y, &z);
        let res_transpose = grad_dot_simd_transpose_x4(&hashes, &x, &y, &z);

        assert_close(res_scalar, 0.0, "Empty scalar");
        assert_close(res_simd, 0.0, "Empty Loop SIMD");
        assert_close(res_transpose, 0.0, "Empty Transpose SIMD");
    }

    #[test]
    fn test_single() {
        let hashes = vec![4];
        let x = vec![1.5];
        let y = vec![-2.0];
        let z = vec![0.5];

        let res_scalar = grad_dot_original(&hashes, &x, &y, &z);
        let res_simd = grad_dot_simd_portable_x4(&hashes, &x, &y, &z);
        let res_transpose = grad_dot_simd_transpose_x4(&hashes, &x, &y, &z);

        // Gradient 4: [1.0, 0.0, 1.0]
        // 1.5 * 1.0 + -2.0 * 0.0 + 0.5 * 1.0 = 1.5 + 0.5 = 2.0
        assert_close(res_scalar, 2.0, "Single scalar");
        assert_close(res_simd, 2.0, "Single Loop SIMD");
        assert_close(res_transpose, 2.0, "Single Transpose SIMD");
    }

    #[test]
    fn test_exact_multiple_of_4() {
        let hashes = vec![0, 1, 2, 3];
        let x = vec![1.0, 1.0, 1.0, 1.0];
        let y = vec![1.0, 1.0, 1.0, 1.0];
        let z = vec![1.0, 1.0, 1.0, 1.0];

        let res_scalar = grad_dot_original(&hashes, &x, &y, &z);
        let res_simd = grad_dot_simd_portable_x4(&hashes, &x, &y, &z);
        let res_transpose = grad_dot_simd_transpose_x4(&hashes, &x, &y, &z);

        assert_close(res_simd, res_scalar, "Exact multiple of 4 (Loop)");
        assert_close(res_transpose, res_scalar, "Exact multiple of 4 (Transpose)");
    }
}
