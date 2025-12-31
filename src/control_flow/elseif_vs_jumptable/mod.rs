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

use crate::registry::{AlgorithmRunner, BenchmarkResult};

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
}
