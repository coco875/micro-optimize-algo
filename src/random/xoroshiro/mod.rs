pub mod bench;
pub mod code;
#[cfg(test)]
pub mod test;

use crate::registry::{AlgorithmRunner, BenchmarkClosure, BenchmarkResult};
use std::cell::RefCell;
use std::hint::black_box;

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

    fn run_benchmarks(&self, size: usize, iterations: usize) -> Vec<BenchmarkResult> {
        // Skip small sizes as they might be too fast/noisy for this throughput benchmark
        if size < 1024 {
            return Vec::new();
        }

        // Benchmark generating 'size' random numbers per iteration
        bench::run_all_benchmarks(size, iterations)
    }

    fn verify(&self) -> Result<(), String> {
        let variants = code::available_variants();

        // Find reference implementation
        let original_variant = variants
            .iter()
            .find(|v| v.name == "original")
            .ok_or("No 'original' variant found for reference")?;

        let seed_lo_ref = 0xdeadbeef;
        let seed_hi_ref = 0xcafebab;

        // Generate a sequence of numbers for reference
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

    fn get_benchmark_closures(&self, size: usize, seed: u64) -> Vec<BenchmarkClosure> {
        use std::time::Instant;

        code::available_variants()
            .into_iter()
            .map(|v| {
                let func = v.function;
                // Each closure has its own RNG state derived from seed
                let state0 = RefCell::new(seed);
                let state1 = RefCell::new(seed.wrapping_mul(0xDEADBEEF));

                BenchmarkClosure {
                    name: v.name,
                    description: v.description,
                    run: Box::new(move || {
                        let mut s0 = state0.borrow_mut();
                        let mut s1 = state1.borrow_mut();
                        let mut result = 0u64;

                        let start = Instant::now();
                        for _ in 0..size {
                            result = func(&mut s0, &mut s1);
                            black_box(result);
                        }
                        let elapsed = start.elapsed();

                        (result as f64, elapsed)
                    }),
                }
            })
            .collect()
    }

    fn warmup(&self, size: usize, warmup_iterations: usize, seed: u64) {
        for v in code::available_variants() {
            let mut s0: u64 = seed;
            let mut s1: u64 = seed.wrapping_mul(0xDEADBEEF);
            for _ in 0..warmup_iterations {
                for _ in 0..size {
                    black_box((v.function)(&mut s0, &mut s1));
                }
            }
        }
    }
}
