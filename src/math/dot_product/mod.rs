//! # Dot Product Algorithm
//!
//! The dot product (also known as scalar product) computes the sum of products
//! of corresponding elements in two vectors:
//!
//! `dot(a, b) = Σ(a[i] * b[i])`

pub mod code;
pub mod test;

pub use code::*;

use crate::registry::{AlgorithmRunner, VariantClosure};
use rand::Rng;
use std::sync::Arc;

/// Runner for the dot product algorithm
pub struct DotProductRunner;

impl AlgorithmRunner for DotProductRunner {
    fn name(&self) -> &'static str {
        "dot_product"
    }

    fn description(&self) -> &'static str {
        "Computes the sum of products of corresponding vector elements"
    }

    fn category(&self) -> &'static str {
        "math"
    }

    fn available_variants(&self) -> Vec<&'static str> {
        code::available_variants().iter().map(|v| v.name).collect()
    }

    fn get_variant_closures<'a>(&'a self, size: usize) -> Vec<VariantClosure<'a>> {
        // Generate test data
        let mut rng = rand::rng();
        let a: Arc<Vec<f32>> = Arc::new((0..size).map(|_| rng.random_range(-1.0..1.0)).collect());
        let b: Arc<Vec<f32>> = Arc::new((0..size).map(|_| rng.random_range(-1.0..1.0)).collect());

        code::available_variants()
            .into_iter()
            .map(|v| {
                let a = Arc::clone(&a);
                let b = Arc::clone(&b);
                let func = v.function;

                VariantClosure {
                    name: v.name,
                    description: v.description,
                    run: Box::new(move || {
                        // Timing inside closure eliminates Fn trait overhead
                        let (elapsed, result) = crate::measure!(func(&a, &b));
                        (elapsed, Some(result as f64))
                    }),
                }
            })
            .collect()
    }

    fn verify(&self) -> Result<(), String> {
        let mut rng = rand::rng();
        let size = 1023;
        let a: Vec<f32> = (0..size).map(|_| rng.random_range(-1.0..1.0)).collect();
        let b: Vec<f32> = (0..size).map(|_| rng.random_range(-1.0..1.0)).collect();

        let variants = code::available_variants();
        let original_variant = variants
            .iter()
            .find(|v| v.name == "original")
            .ok_or("No 'original' variant found for reference")?;

        let expected = (original_variant.function)(&a, &b);

        for variant in &variants {
            if variant.name == "original" {
                continue;
            }

            let result = (variant.function)(&a, &b);
            let diff = (result - expected).abs();

            if diff > 1e-4 {
                return Err(format!(
                    "Variant '{}' failed verification. Expected {}, got {}, diff {}",
                    variant.name, expected, result, diff
                ));
            }
        }

        Ok(())
    }
}
