use super::code;

#[test]
fn test_xoroshiro_known_value() {
    // Test with a simple known seed to verify the algorithm logic
    // Seed: s0 = 1, s1 = 0
    //
    // Algorithm Xoroshiro128++:
    // result = rotl(s0 + s1, 17) + s0
    //
    // Iteration 1:
    // s0 = 1, s1 = 0
    // sum = 1
    // rotl(1, 17) = 1 << 17 = 131072
    // result = 131072 + 1 = 131073

    let mut s0 = 1;
    let mut s1 = 0;

    let result = code::xoroshiro_original(&mut s0, &mut s1);
    assert_eq!(
        result, 131073,
        "First generated number should be 131073 for seed (1, 0)"
    );
}

#[test]
fn test_xoroshiro_determinism() {
    let variants = code::available_variants();

    for variant in variants {
        let mut s0_a = 0x12345678;
        let mut s1_a = 0x87654321;

        let mut s0_b = 0x12345678;
        let mut s1_b = 0x87654321;

        // Run two identical sequences
        for _ in 0..100 {
            let res_a = (variant.function)(&mut s0_a, &mut s1_a);
            let res_b = (variant.function)(&mut s0_b, &mut s1_b);
            assert_eq!(
                res_a, res_b,
                "Variant {} should be deterministic",
                variant.name
            );
        }
    }
}

#[test]
fn test_all_variants_match_original() {
    let variants = code::available_variants();
    let original = variants
        .iter()
        .find(|v| v.name == "original")
        .expect("original variant not found");

    let seed_lo = 0xdeadbeef;
    let seed_hi = 0xcafebab;

    // Generate reference sequence
    let mut expected = Vec::new();
    let mut s0 = seed_lo;
    let mut s1 = seed_hi;
    for _ in 0..1000 {
        expected.push((original.function)(&mut s0, &mut s1));
    }

    for variant in &variants {
        if variant.name == "original" {
            continue;
        }

        let mut s0 = seed_lo;
        let mut s1 = seed_hi;

        for (i, &exp) in expected.iter().enumerate() {
            let got = (variant.function)(&mut s0, &mut s1);
            assert_eq!(got, exp, "Variant {} mismatch at index {}", variant.name, i);
        }
    }
}
