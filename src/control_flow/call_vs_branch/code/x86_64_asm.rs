//! x86_64 Assembly implementations showing the difference between:
//! - Function calls (CALL/RET) with stack operations
//! - Inline code without call overhead
//!
//! # Function Call Assembly (CALL/RET)
//! ```asm
//! process_with_calls:
//!     ; Save callee-saved registers if needed
//!     call double         ; Push return address, jump to double
//!                         ; ... double executes, returns with RET
//!     call add_ten        ; Push return address, jump to add_ten
//!                         ; ... add_ten executes, returns with RET
//!     call square         ; Push return address, jump to square
//!                         ; ... square executes, returns with RET
//!     ret
//!
//! double:
//!     add edi, edi        ; x * 2
//!     mov eax, edi
//!     ret                 ; Pop return address, jump back
//!
//! add_ten:
//!     add edi, 10
//!     mov eax, edi
//!     ret
//!
//! square:
//!     imul edi, edi
//!     mov eax, edi
//!     ret
//! ```
//!
//! # Inline Assembly (no CALL)
//! ```asm
//! process_inline:
//!     add edi, edi        ; step1 = x * 2
//!     add edi, 10         ; step2 = step1 + 10
//!     imul eax, edi, edi  ; step3 = step2 * step2
//!     ret
//! ```
//!
//! # Performance Comparison
//!
//! | Aspect | CALL/RET | Inline |
//! |--------|----------|--------|
//! | Overhead per call | ~3-5 cycles | 0 |
//! | Stack operations | Push/Pop return address | None |
//! | Code size | Smaller (shared code) | Larger (duplicated) |
//! | I-cache | Better for large functions | May cause pressure |
//! | Return prediction | Uses RSB (Return Stack Buffer) | N/A |
//!
//! # Return Stack Buffer (RSB)
//!
//! Modern CPUs have a hardware stack that predicts return addresses.
//! Each CALL pushes onto RSB, each RET pops from RSB.
//! Misprediction (e.g., from ROP attacks or unusual patterns) is costly (~15-20 cycles).

use std::arch::asm;

/// Process using simulated function calls via inline assembly
///
/// This demonstrates CALL/RET overhead. We use local labels to simulate
/// function calls within the same inline asm block.
///
/// Note: True separate function calls would require extern functions,
/// but this demonstrates the CALL/RET mechanism.
#[inline(never)]
pub fn process_with_calls(value: u32) -> u32 {
    let result: u32;
    
    unsafe {
        asm!(
            // Main function body
            "mov {val:e}, {input:e}",
            
            // Call "double" subroutine
            "call 20f",
            
            // Call "add_ten" subroutine  
            "call 30f",
            
            // Call "square" subroutine
            "call 40f",
            
            // Done - result is in val
            "jmp 99f",
            
            // === Subroutine: double ===
            // Input: val, Output: val = val * 2
            "20:",
            "add {val:e}, {val:e}",
            "ret",
            
            // === Subroutine: add_ten ===
            // Input: val, Output: val = val + 10
            "30:",
            "add {val:e}, 10",
            "ret",
            
            // === Subroutine: square ===
            // Input: val, Output: val = val * val
            "40:",
            "imul {val:e}, {val:e}",
            "ret",
            
            // === End ===
            "99:",
            
            input = in(reg) value,
            val = out(reg) result,
            options(nostack),
        );
    }
    
    result
}

/// Process with everything inlined - no CALL/RET overhead
///
/// All operations are sequential, no stack manipulation needed.
#[inline(never)]
pub fn process_inline(value: u32) -> u32 {
    let result: u32;
    
    unsafe {
        asm!(
            // Step 1: double (val * 2)
            "mov {val:e}, {input:e}",
            "add {val:e}, {val:e}",
            
            // Step 2: add_ten (val + 10)
            "add {val:e}, 10",
            
            // Step 3: square (val * val)
            "imul {val:e}, {val:e}",
            
            input = in(reg) value,
            val = out(reg) result,
            options(nostack, nomem, pure),
        );
    }
    
    result
}

/// Process using conditional BRANCHES (JMP/Jcc instructions)
///
/// Uses unconditional jumps (JMP) to simulate subroutine calls without
/// the CALL/RET overhead. This is like inlining but with jumps.
///
/// This demonstrates the cost of branch instructions themselves
/// (pipeline stalls, potential mispredictions) without CALL overhead.
///
/// ```asm
/// process_with_branch:
///     jmp .do_double      ; Jump to double code
/// .after_double:
///     jmp .do_add_ten     ; Jump to add_ten code  
/// .after_add_ten:
///     jmp .do_square      ; Jump to square code
/// .after_square:
///     ret
///
/// .do_double:
///     add eax, eax
///     jmp .after_double   ; Jump back (like a branch-based return)
/// ; ...
/// ```
#[inline(never)]
pub fn process_with_branch(value: u32) -> u32 {
    let result: u32;
    
    unsafe {
        asm!(
            // Main function body
            "mov {val:e}, {input:e}",
            
            // Jump to "double" code block
            "jmp 20f",
            "21:",  // Return point after double
            
            // Jump to "add_ten" code block
            "jmp 30f",
            "31:",  // Return point after add_ten
            
            // Jump to "square" code block
            "jmp 40f",
            "41:",  // Return point after square
            
            // Done - result is in val
            "jmp 99f",
            
            // === Code block: double ===
            "20:",
            "add {val:e}, {val:e}",
            "jmp 21b",  // Branch back (unconditional jump)
            
            // === Code block: add_ten ===
            "30:",
            "add {val:e}, 10",
            "jmp 31b",  // Branch back
            
            // === Code block: square ===
            "40:",
            "imul {val:e}, {val:e}",
            "jmp 41b",  // Branch back
            
            // === End ===
            "99:",
            
            input = in(reg) value,
            val = out(reg) result,
            options(nostack, nomem),
        );
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expected_result(value: u32) -> u32 {
        let step1 = value.wrapping_mul(2);      // double
        let step2 = step1.wrapping_add(10);     // add_ten
        step2.wrapping_mul(step2)               // square
    }

    #[test]
    fn test_process_with_calls() {
        for v in [0, 1, 5, 10, 100, 1000] {
            assert_eq!(
                process_with_calls(v), 
                expected_result(v),
                "process_with_calls({}) failed", v
            );
        }
    }

    #[test]
    fn test_process_inline() {
        for v in [0, 1, 5, 10, 100, 1000] {
            assert_eq!(
                process_inline(v), 
                expected_result(v),
                "process_inline({}) failed", v
            );
        }
    }

    #[test]
    fn test_process_with_branch() {
        for v in [0, 1, 5, 10, 100, 1000] {
            assert_eq!(
                process_with_branch(v), 
                expected_result(v),
                "process_with_branch({}) failed", v
            );
        }
    }

    #[test]
    fn test_all_match() {
        for v in 0..1000 {
            let expected = expected_result(v);
            assert_eq!(process_with_calls(v), expected, "process_with_calls mismatch for {}", v);
            assert_eq!(process_with_branch(v), expected, "process_with_branch mismatch for {}", v);
            assert_eq!(process_inline(v), expected, "process_inline mismatch for {}", v);
        }
    }
}
