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

pub mod code;
pub mod bench;
pub mod test;

pub use code::*;

use crate::registry::{AlgorithmRunner, BenchmarkResult};
use rand::Rng;

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
        code::available_variants()
            .iter()
            .map(|v| v.name)
            .collect()
    }
    
    fn run_benchmarks(&self, size: usize, iterations: usize) -> Vec<BenchmarkResult> {
        let mut rng = rand::thread_rng();
        let a: Vec<f32> = (0..size).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let b: Vec<f32> = (0..size).map(|_| rng.gen_range(-1.0..1.0)).collect();
        
        bench::run_all_benchmarks(&a, &b, iterations)
            .into_iter()
            .map(|r| BenchmarkResult {
                variant_name: r.name,
                description: r.description,
                avg_time: r.avg_time,
                min_time: r.min_time,
                max_time: r.max_time,
                std_dev: r.std_dev,
                iterations,
                result_sample: r.result as f64,
                compiler: r.compiler,
            })
            .collect()
    }

    fn verify(&self) -> Result<(), String> {
        let mut rng = rand::thread_rng();
        // Use a non-aligned size to test edge cases
        let size = 1023;
        let a: Vec<f32> = (0..size).map(|_| rng.gen_range(-1.0..1.0)).collect();
        let b: Vec<f32> = (0..size).map(|_| rng.gen_range(-1.0..1.0)).collect();
        
        // Find reference implementation (assumed to be named "original")
        let variants = code::available_variants();
        let original_variant = variants.iter()
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
}
