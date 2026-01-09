//! # Dot Product Algorithm
//!
//! The dot product (also known as scalar product) computes the sum of products
//! of corresponding elements in two vectors:
//!
//! `dot(a, b) = Σ(a[i] * b[i])`
//!
//! ## Optimization Strategies
//!
//! - **Loop unrolling**: Process multiple elements per iteration to reduce loop overhead
//! - **SIMD**: Use vector instructions (AVX2, SSE2, NEON) to process 4-8 floats simultaneously
//! - **Cache optimization**: Ensure sequential memory access patterns
//! - **FMA**: Use fused multiply-add instructions when available

pub mod bench;
pub mod code;
pub mod test;

pub use code::*;

use crate::registry::{AlgorithmRunner, BenchmarkClosure, BenchmarkResult};
use crate::utils::bench::SeededRng;
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

    fn run_benchmarks(&self, size: usize, iterations: usize) -> Vec<BenchmarkResult> {
        let mut rng = rand::thread_rng();
        let a: Vec<f32> = (0..size).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let b: Vec<f32> = (0..size).map(|_| rng.gen_range(-1.0..1.0)).collect();

        bench::run_all_benchmarks(&a, &b, iterations)
    }

    fn verify(&self) -> Result<(), String> {
        let mut rng = rand::thread_rng();
        // Use a non-aligned size to test edge cases
        let size = 1023;
        let a: Vec<f32> = (0..size).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let b: Vec<f32> = (0..size).map(|_| rng.gen_range(-1.0..1.0)).collect();

        // Find reference implementation (assumed to be named "original")
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

            // Allow small specific tolerance for floating point accumulation differences
            // Dot product accumulation order affects lower bits
            if diff > 1e-4 {
                return Err(format!(
                    "Variant '{}' failed verification. Expected {}, got {}, diff {}",
                    variant.name, expected, result, diff
                ));
            }
        }

        Ok(())
    }

    fn get_benchmark_closures(&self, size: usize, seed: u64) -> Vec<BenchmarkClosure> {
        use std::time::Instant;

        let mut rng = SeededRng::new(seed);
        let a: Arc<Vec<f32>> = Arc::new((0..size).map(|_| rng.next_f32_range()).collect());
        let b: Arc<Vec<f32>> = Arc::new((0..size).map(|_| rng.next_f32_range()).collect());

        code::available_variants()
            .into_iter()
            .map(|v| {
                let a = Arc::clone(&a);
                let b = Arc::clone(&b);
                let func = v.function;

                BenchmarkClosure {
                    name: v.name,
                    description: v.description,
                    compiler: v.compiler,
                    run: Box::new(move || {
                        let start = Instant::now();
                        let result = func(&a, &b);
                        let elapsed = start.elapsed();
                        (std::hint::black_box(result) as f64, elapsed)
                    }),
                }
            })
            .collect()
    }

    fn warmup(&self, size: usize, warmup_iterations: usize, seed: u64) {
        let mut rng = SeededRng::new(seed);
        let a: Vec<f32> = (0..size).map(|_| rng.next_f32_range()).collect();
        let b: Vec<f32> = (0..size).map(|_| rng.next_f32_range()).collect();

        for v in code::available_variants() {
            for _ in 0..warmup_iterations {
                std::hint::black_box((v.function)(&a, &b));
            }
        }
    }
}
