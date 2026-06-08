//! Original Rust implementation using function calls
//!
//! This serves as the reference implementation. The Rust compiler may or may not
//! inline these functions depending on optimization level and heuristics.
//! Use `#[inline(never)]` to force function calls for fair comparison.

/// Helper function - doubles the value (forced to not inline)
#[inline(never)]
fn double(x: u32) -> u32 {
    x * 2
}

/// Helper function - adds 10 (forced to not inline)
#[inline(never)]
fn add_ten(x: u32) -> u32 {
    x + 10
}

/// Helper function - squares the value (forced to not inline)
#[inline(never)]
fn square(x: u32) -> u32 {
    x.wrapping_mul(x)
}

use crate::control_flow::call_vs_branch::CallVsBranchVariant;
use crate::variant;

#[variant(group = "call_vs_branch")]
pub struct Original;

impl CallVsBranchVariant for Original {
    fn name() -> &'static str {
        "original"
    }
    fn compute(val: u32) -> u32 {
        let step1 = double(val); // CALL double, RET
        let step2 = add_ten(step1); // CALL add_ten, RET
        square(step2) // CALL square, RET
    }
}

#[variant(group = "call_vs_branch")]
pub struct RustInline;

impl CallVsBranchVariant for RustInline {
    fn name() -> &'static str {
        "rust_inline"
    }
    fn compute(val: u32) -> u32 {
        let step1 = val * 2;
        let step2 = step1 + 10;
        step2.wrapping_mul(step2)
    }
}
