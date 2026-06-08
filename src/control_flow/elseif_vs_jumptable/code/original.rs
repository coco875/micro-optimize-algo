//! Original Rust implementation using match expression
//!
//! The compiler may optimize this to:
//! - Jump table (for dense, contiguous cases)
//! - Binary search (for sparse cases)
//! - Linear if-else chain (for few cases)

use crate::control_flow::elseif_vs_jumptable::ElseIfVsJumpTableVariant;
use crate::variant;

#[variant(group = "elseif_vs_jumptable")]
pub struct Original;

impl ElseIfVsJumpTableVariant for Original {
    fn name() -> &'static str {
        "original"
    }
    fn compute(op: u8, val: u32) -> u32 {
        match op {
            0 => val,
            1 => val * 2,
            2 => val * 3,
            3 => val * 4,
            4 => val * 5,
            5 => val * 6,
            6 => val * 7,
            7 => val * 8,
            _ => 0, // Invalid opcode
        }
    }
}
