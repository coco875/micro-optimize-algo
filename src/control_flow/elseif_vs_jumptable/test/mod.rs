//! Tests for else-if vs jump table implementations

use super::code::{get_variants, original};

/// Verify all variants produce the same results as the original
pub fn verify_all() -> Result<(), String> {
    let test_cases: Vec<(u8, u32)> = vec![
        (0, 1),
        (0, 100),
        (1, 1),
        (1, 50),
        (2, 1),
        (2, 33),
        (3, 1),
        (3, 25),
        (4, 1),
        (4, 20),
        (5, 1),
        (5, 16),
        (6, 1),
        (6, 14),
        (7, 1),
        (7, 12),
        (8, 100),   // Invalid
        (255, 100), // Invalid
    ];

    for variant in get_variants() {
        if variant.name == "original" {
            continue;
        }

        for &(opcode, value) in &test_cases {
            let expected = original::dispatch_operation(opcode, value);
            let actual = (variant.function)(opcode, value);

            if actual != expected {
                return Err(format!(
                    "Variant '{}' failed for opcode={}, value={}: expected {}, got {}",
                    variant.name, opcode, value, expected, actual
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
    fn test_all_opcodes() {
        let variants = get_variants();
        let value = 12u32;

        for variant in &variants {
            // Test each valid opcode
            assert_eq!((variant.function)(0, value), 12, "{}: op 0", variant.name);
            assert_eq!((variant.function)(1, value), 24, "{}: op 1", variant.name);
            assert_eq!((variant.function)(2, value), 36, "{}: op 2", variant.name);
            assert_eq!((variant.function)(3, value), 48, "{}: op 3", variant.name);
            assert_eq!((variant.function)(4, value), 60, "{}: op 4", variant.name);
            assert_eq!((variant.function)(5, value), 72, "{}: op 5", variant.name);
            assert_eq!((variant.function)(6, value), 84, "{}: op 6", variant.name);
            assert_eq!((variant.function)(7, value), 96, "{}: op 7", variant.name);

            // Test invalid opcodes
            assert_eq!(
                (variant.function)(8, value),
                0,
                "{}: invalid op",
                variant.name
            );
        }
    }

    #[test]
    fn test_edge_values() {
        let variants = get_variants();

        for variant in &variants {
            // Test with 0
            assert_eq!(
                (variant.function)(0, 0),
                0,
                "{}: 0 * anything = 0",
                variant.name
            );

            // Test with 1
            assert_eq!(
                (variant.function)(0, 1),
                1,
                "{}: identity of 1",
                variant.name
            );

            // Test with large value (avoiding overflow)
            assert_eq!(
                (variant.function)(1, 1000000),
                2000000,
                "{}: large value",
                variant.name
            );
        }
    }
}
