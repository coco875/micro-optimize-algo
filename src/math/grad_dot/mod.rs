//! # Gradient Dot Product Algorithm
//!
//! Computes the dot product of Perlin gradient vectors and position vectors
//! over slices of inputs.
//!
//! `grad_dot(hash, x, y, z) = g[0]*x + g[1]*y + g[2]*z`

pub mod code;
pub mod test;

pub use code::*;

use crate::registry::{AlgorithmRunner, VariantClosure};
use rand::Rng;
use std::sync::Arc;

/// Runner for the gradient dot product algorithm
pub struct GradDotRunner;

impl AlgorithmRunner for GradDotRunner {
    fn name(&self) -> &'static str {
        "grad_dot"
    }

    fn description(&self) -> &'static str {
        "Computes Perlin gradient dot products over slices of positions"
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
        let hashes: Arc<Vec<usize>> = Arc::new((0..size).map(|_| rng.random_range(0..256)).collect());
        let x: Arc<Vec<f64>> = Arc::new((0..size).map(|_| rng.random_range(-1.0..1.0)).collect());
        let y: Arc<Vec<f64>> = Arc::new((0..size).map(|_| rng.random_range(-1.0..1.0)).collect());
        let z: Arc<Vec<f64>> = Arc::new((0..size).map(|_| rng.random_range(-1.0..1.0)).collect());

        code::available_variants()
            .into_iter()
            .map(|v| {
                let hashes = Arc::clone(&hashes);
                let x = Arc::clone(&x);
                let y = Arc::clone(&y);
                let z = Arc::clone(&z);
                let func = v.function;

                VariantClosure {
                    name: v.name,
                    description: v.description,
                    run: Box::new(move || {
                        // Timing inside closure eliminates Fn trait overhead
                        let (elapsed, result) = crate::measure!(func(&hashes, &x, &y, &z));
                        (elapsed, Some(result))
                    }),
                }
            })
            .collect()
    }

    fn verify(&self) -> Result<(), String> {
        let mut rng = rand::rng();
        let size = 1023;
        let hashes: Vec<usize> = (0..size).map(|_| rng.random_range(0..256)).collect();
        let x: Vec<f64> = (0..size).map(|_| rng.random_range(-1.0..1.0)).collect();
        let y: Vec<f64> = (0..size).map(|_| rng.random_range(-1.0..1.0)).collect();
        let z: Vec<f64> = (0..size).map(|_| rng.random_range(-1.0..1.0)).collect();

        let variants = code::available_variants();
        let original_variant = variants
            .iter()
            .find(|v| v.name == "original")
            .ok_or("No 'original' variant found for reference")?;

        let expected = (original_variant.function)(&hashes, &x, &y, &z);

        for variant in &variants {
            if variant.name == "original" {
                continue;
            }

            let result = (variant.function)(&hashes, &x, &y, &z);
            let diff = (result - expected).abs();

            if diff > 1e-9 {
                return Err(format!(
                    "Variant '{}' failed verification. Expected {}, got {}, diff {}",
                    variant.name, expected, result, diff
                ));
            }
        }

        Ok(())
    }
}
