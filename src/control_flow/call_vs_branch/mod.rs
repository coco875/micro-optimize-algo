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

use crate::registry::{AlgorithmRunner, BenchmarkResult};

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
}
