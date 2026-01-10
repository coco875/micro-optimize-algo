//! Test utilities for dot product implementations.

#[cfg(test)]
mod tests {
    use crate::math::dot_product::code::*;

    const EPSILON: f32 = 1e-5;

    fn assert_close(a: f32, b: f32, msg: &str) {
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
    fn test_original_basic() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [5.0, 6.0, 7.0, 8.0];
        // 1*5 + 2*6 + 3*7 + 4*8 = 5 + 12 + 21 + 32 = 70
        let result = dot_product_original(&a, &b);
        assert_close(result, 70.0, "original basic");
    }

    #[test]
    fn test_original_empty() {
        let a: [f32; 0] = [];
        let b: [f32; 0] = [];
        let result = dot_product_original(&a, &b);
        assert_close(result, 0.0, "original empty");
    }

    #[test]
    fn test_original_single() {
        let a = [3.0];
        let b = [4.0];
        let result = dot_product_original(&a, &b);
        assert_close(result, 12.0, "original single");
    }

    // Variant testing is now handled by the generic verify() method via the Registry.
}
