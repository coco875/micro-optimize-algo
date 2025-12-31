pub mod code;
pub mod bench;
#[cfg(test)]
pub mod test;

use crate::registry::{AlgorithmRunner, BenchmarkResult};

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
        code::available_variants()
            .iter()
            .map(|v| v.name)
            .collect()
    }

    fn run_benchmarks(&self, size: usize, iterations: usize) -> Vec<BenchmarkResult> {
        // Skip small sizes as they might be too fast/noisy for this throughput benchmark
        if size < 1024 {
            return Vec::new();
        }

        // Benchmark generating 'size' random numbers per iteration
        bench::run_all_benchmarks(size, iterations)
            .into_iter()
            .map(|r| BenchmarkResult {
                variant_name: r.name,
                description: r.description,
                avg_time: r.avg_time,
                min_time: r.min_time,
                max_time: r.max_time,
                std_dev: r.std_dev,
                iterations,
                result_sample: r.result as f64, // Cast u64 to f64 for generic display
                compiler: r.compiler,
            })
            .collect()
    }

    fn verify(&self) -> Result<(), String> {
        let variants = code::available_variants();
        
        // Find reference implementation
        let original_variant = variants.iter()
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
}
