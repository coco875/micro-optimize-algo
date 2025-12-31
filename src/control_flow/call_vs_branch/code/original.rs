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

/// Process a value through a chain of function calls:
/// result = square(add_ten(double(value)))
///
/// Each step is a separate function call with CALL/RET overhead.
#[inline(never)]
pub fn process_with_calls(value: u32) -> u32 {
    let step1 = double(value);     // CALL double, RET
    let step2 = add_ten(step1);    // CALL add_ten, RET
    let step3 = square(step2);     // CALL square, RET
    step3
}

/// Process a value with everything inlined (for comparison reference)
#[inline(never)]
pub fn process_inline(value: u32) -> u32 {
    let step1 = value * 2;
    let step2 = step1 + 10;
    let step3 = step2.wrapping_mul(step2);
    step3
}
