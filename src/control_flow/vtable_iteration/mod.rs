//! # VTable Iteration Benchmark
//!
//! Compare different iteration methods over a pool of polymorphic objects
//! with manual vtables (C repr).

pub mod code;
pub mod test;

use crate::registry::{AlgorithmRunner, VariantClosure};
use code::get_variants;
use std::cell::RefCell;

pub struct VTableIterationRunner;

impl AlgorithmRunner for VTableIterationRunner {
    fn name(&self) -> &'static str {
        "vtable_iteration"
    }

    fn category(&self) -> &'static str {
        "control_flow"
    }

    fn description(&self) -> &'static str {
        "Compare iteration methods over object pools with manual vtables"
    }

    fn available_variants(&self) -> Vec<&'static str> {
        get_variants().iter().map(|v| v.name).collect()
    }

    fn get_variant_closures<'a>(&'a self, size: usize) -> Vec<VariantClosure<'a>> {
        let seed = 0x12345678u64;
        let variants = get_variants();

        variants
            .into_iter()
            .map(|variant| {
                let pool = RefCell::new((variant.create)(size, seed));
                VariantClosure {
                    name: variant.name,
                    description: variant.description,
                    run: Box::new(move || {
                        let (elapsed, result) = crate::measure!({
                            pool.borrow_mut().iterate()
                        });
                        (elapsed, Some(result as f64))
                    }),
                }
            })
            .collect()
    }

    fn verify(&self) -> Result<(), String> {
        test::verify_all()
    }
}
