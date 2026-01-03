//! # Else-If Chain vs Jump Table Comparison
//!
//! This module demonstrates the difference between a chain of else-if statements
//! and a jump table (also known as branch table or switch table) in x86_64 assembly.
//!
//! ## Key Concepts
//!
//! - **Else-If Chain**: Linear comparison, O(n) time complexity
//! - **Jump Table**: Indexed lookup, O(1) time complexity
//!
//! ## Performance Implications
//!
//! - Else-If: Time grows with position in chain (early cases faster)
//! - Jump Table: Constant time regardless of which case is taken
//! - Jump Table has setup overhead (bounds check, address calculation)
//! - For small number of cases (<4), else-if may be faster

pub mod code;
pub mod bench;
pub mod test;

use crate::registry::{AlgorithmRunner, BenchmarkResult, BenchmarkClosure};
use std::hint::black_box;
use std::sync::Arc;

/// Generate test data - random opcodes (0-7) and values
fn generate_test_data(size: usize, seed: u64) -> Vec<(u8, u32)> {
    let mut data = Vec::with_capacity(size);
    let mut rng = seed;
    
    for _ in 0..size {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let opcode = ((rng >> 32) % 8) as u8;
        let value = ((rng >> 40) % 1000) as u32 + 1;
        data.push((opcode, value));
    }
    data
}

pub struct ElseIfVsJumpTableRunner;

impl AlgorithmRunner for ElseIfVsJumpTableRunner {
    fn name(&self) -> &'static str {
        "elseif_vs_jumptable"
    }

    fn category(&self) -> &'static str {
        "control_flow"
    }

    fn description(&self) -> &'static str {
        "Comparison between else-if chains and jump tables in x86_64 assembly"
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
        
        let data: Arc<Vec<(u8, u32)>> = Arc::new(generate_test_data(size, seed));
        
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
                        for &(op, val) in data.iter() {
                            last_result = black_box(func(black_box(op), black_box(val)));
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
                for &(op, val) in data.iter().take(100) {
                    black_box((v.func)(black_box(op), black_box(val)));
                }
            }
        }
    }
}
