//! Original Rust implementation using match expression
//!
//! The compiler may optimize this to:
//! - Jump table (for dense, contiguous cases)
//! - Binary search (for sparse cases)
//! - Linear if-else chain (for few cases)

/// Simulates an opcode dispatcher that multiplies a value by different constants
/// based on the operation code.
///
/// Opcodes:
/// - 0: identity (×1)
/// - 1: double (×2)
/// - 2: triple (×3)
/// - 3: quadruple (×4)
/// - 4: ×5
/// - 5: ×6
/// - 6: ×7
/// - 7: ×8
/// - _: zero (invalid opcode)
#[inline(never)]
pub fn dispatch_operation(opcode: u8, value: u32) -> u32 {
    match opcode {
        0 => value,
        1 => value * 2,
        2 => value * 3,
        3 => value * 4,
        4 => value * 5,
        5 => value * 6,
        6 => value * 7,
        7 => value * 8,
        _ => 0, // Invalid opcode
    }
}
