//! # Call vs Branch Comparison
//!
//! This module demonstrates the difference between function calls (CALL/RET)
//! and inline conditional branches in x86_64 assembly.
//!
//! ## Key Concepts
//!
//! - **CALL**: Pushes return address on stack, jumps to function, RET pops and returns.
//!   Has overhead from stack operations and return address prediction.
//!
//! - **Inline Branch (Jcc)**: Code is inlined, no function call overhead.
//!   Larger code size but faster execution for small operations.
//!
//! ## Performance Implications
//!
//! - CALL: ~3-5 cycles overhead for call/ret pair on modern CPUs
//! - Inline: No call overhead, but may increase code size and I-cache pressure
//! - Return Stack Buffer (RSB): CPUs predict return addresses, misprediction is costly

pub mod code;
pub mod bench;
pub mod test;

use crate::registry::{AlgorithmRunner, BenchmarkResult, BenchmarkClosure};
use std::hint::black_box;
use std::sync::Arc;

/// Generate test data - random values to stress branch prediction
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
        "Comparison between function calls (CALL/RET) and inline code in x86_64 assembly"
    }

    fn available_variants(&self) -> Vec<&'static str> {
        code::get_variants().iter().map(|v| v.name).collect()
    }

    fn run_benchmarks(&self, size: usize, iterations: usize) -> Vec<BenchmarkResult> {
        bench::run_benchmarks(size, iterations)
    }

    fn verify(&self) -> Result<(), String> {
        test::verify_all()
    }
    
    fn get_benchmark_closures(&self, size: usize, seed: u64) -> Vec<BenchmarkClosure> {
        use std::time::Instant;
        
        let data: Arc<Vec<u32>> = Arc::new(generate_test_data(size, seed));
        
        code::get_variants()
            .into_iter()
            .map(|v| {
                let data = Arc::clone(&data);
                let func = v.func;
                
                BenchmarkClosure {
                    name: v.name,
                    description: v.description,
                    compiler: None,
                    run: Box::new(move || {
                        let mut last_result = 0u32;
                        
                        let start = Instant::now();
                        for &v in data.iter() {
                            last_result = black_box(func(black_box(v)));
                        }
                        let elapsed = start.elapsed();
                        
                        (last_result as f64, elapsed)
                    }),
                }
            })
            .collect()
    }
    
    fn warmup(&self, size: usize, warmup_iterations: usize, seed: u64) {
        let data = generate_test_data(size, seed);
        
        for v in code::get_variants() {
            for _ in 0..warmup_iterations {
                for &val in data.iter().take(100) {
                    black_box((v.func)(black_box(val)));
                }
            }
        }
    }
}
