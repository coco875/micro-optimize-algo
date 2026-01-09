//! Tests for call vs branch implementations

use super::code::{get_variants, original};

/// Verify all variants produce the same results as the original
pub fn verify_all() -> Result<(), String> {
    let test_values: Vec<u32> = vec![0, 1, 2, 5, 10, 50, 100, 255, 500, 1000, 10000];

    for variant in get_variants() {
        if variant.name == "original" {
            continue;
        }

        for &value in &test_values {
            let expected = original::process_with_calls(value);
            let actual = (variant.function)(value);

            if actual != expected {
                return Err(format!(
                    "Variant '{}' failed for value {}: expected {}, got {}",
                    variant.name, value, expected, actual
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_variants() {
        verify_all().expect("All variants should produce correct results");
    }

    #[test]
    fn test_expected_computation() {
        // Verify the computation: square(add_ten(double(x)))
        // For x = 5: double(5) = 10, add_ten(10) = 20, square(20) = 400
        let variants = get_variants();
        for variant in &variants {
            assert_eq!(
                (variant.function)(5),
                400,
                "{}: process(5) should be 400",
                variant.name
            );
        }
    }

    #[test]
    fn test_zero() {
        // For x = 0: double(0) = 0, add_ten(0) = 10, square(10) = 100
        let variants = get_variants();
        for variant in &variants {
            assert_eq!(
                (variant.function)(0),
                100,
                "{}: process(0) should be 100",
                variant.name
            );
        }
    }
}
