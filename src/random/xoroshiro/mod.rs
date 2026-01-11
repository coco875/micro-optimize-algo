pub mod code;
#[cfg(test)]
pub mod test;

use crate::registry::{AlgorithmRunner, VariantClosure};

pub struct XoroshiroRunner;

impl AlgorithmRunner for XoroshiroRunner {
    fn name(&self) -> &'static str {
        "xoroshiro128++"
    }

    fn description(&self) -> &'static str {
        "Xoroshiro128++ pseudo-random number generator"
    }

    fn category(&self) -> &'static str {
        "random"
    }

    fn available_variants(&self) -> Vec<&'static str> {
        code::available_variants().iter().map(|v| v.name).collect()
    }

    fn get_variant_closures<'a>(&'a self, size: usize) -> Vec<VariantClosure<'a>> {
        // Only run for the smallest size to avoid redundant measurements
        // Since we measure a single function call, size is irrelevant
        if size != 64 {
            return Vec::new();
        }

        code::available_variants()
            .into_iter()
            .map(|v| {
                let func = v.function;
                // Use mutable captures directly - FnMut allows this
                let mut s0 = 0x12345678u64;
                let mut s1 = 0x87654321u64;

                VariantClosure {
                    name: v.name,
                    description: v.description,
                    run: Box::new(move || {
                        // Timing inside closure eliminates Fn trait overhead
                        let (elapsed, result) = crate::measure!(func(&mut s0, &mut s1));
                        (elapsed, Some(result as f64))
                    }),
                }
            })
            .collect()
    }

    fn verify(&self) -> Result<(), String> {
        let variants = code::available_variants();

        let original_variant = variants
            .iter()
            .find(|v| v.name == "original")
            .ok_or("No 'original' variant found for reference")?;

        let seed_lo_ref = 0xdeadbeef;
        let seed_hi_ref = 0xcafebab;

        let mut expected_sequence = Vec::new();
        let mut s0 = seed_lo_ref;
        let mut s1 = seed_hi_ref;
        for _ in 0..100 {
            expected_sequence.push((original_variant.function)(&mut s0, &mut s1));
        }

        for variant in &variants {
            if variant.name == "original" {
                continue;
            }

            let mut s0 = seed_lo_ref;
            let mut s1 = seed_hi_ref;

            for (i, &expected) in expected_sequence.iter().enumerate() {
                let result = (variant.function)(&mut s0, &mut s1);
                if result != expected {
                    return Err(format!(
                        "Variant '{}' failed verification at iteration {}. Expected {}, got {}",
                        variant.name, i, expected, result
                    ));
                }
            }
        }

        Ok(())
    }
}
