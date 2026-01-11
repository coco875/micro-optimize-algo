//! # Call vs Branch Comparison

pub mod code;
pub mod test;

use crate::registry::{AlgorithmRunner, VariantClosure};
use std::sync::Arc;

/// Generate test data
fn generate_test_data(size: usize, seed: u64) -> Vec<u32> {
    let mut data = Vec::with_capacity(size);
    let mut rng = seed;

    for _ in 0..size {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        data.push((rng >> 32) as u32 % 512);
    }
    data
}

pub struct CallVsBranchRunner;

impl AlgorithmRunner for CallVsBranchRunner {
    fn name(&self) -> &'static str {
        "call_vs_branch"
    }

    fn category(&self) -> &'static str {
        "control_flow"
    }

    fn description(&self) -> &'static str {
        "Comparison between function calls (CALL/RET) and inline code"
    }

    fn available_variants(&self) -> Vec<&'static str> {
        code::get_variants().iter().map(|v| v.name).collect()
    }

    fn get_variant_closures<'a>(&'a self, size: usize) -> Vec<VariantClosure<'a>> {
        let data: Arc<Vec<u32>> = Arc::new(generate_test_data(size, 0x12345678));

        code::get_variants()
            .into_iter()
            .map(|v| {
                let data = Arc::clone(&data);
                let func = v.function;

                VariantClosure {
                    name: v.name,
                    description: v.description,
                    run: Box::new(move || {
                        // Timing inside closure - measures entire loop
                        let (elapsed, _) = crate::measure!({
                            let mut last_result = 0u32;
                            for &val in data.iter() {
                                last_result = std::hint::black_box(func(std::hint::black_box(val)));
                            }
                            last_result
                        });
                        (elapsed, None) // No precision measurement for control flow
                    }),
                }
            })
            .collect()
    }

    fn verify(&self) -> Result<(), String> {
        test::verify_all()
    }
}
