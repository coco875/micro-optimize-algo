#![cfg(target_arch = "x86_64")]
//! x86_64 Assembly implementations showing the difference between:
//! - Function calls (CALL/RET) with stack operations
//! - Inline code without call overhead

use crate::control_flow::call_vs_branch::CallVsBranchVariant;
use crate::variant;
use std::arch::asm;

#[variant(group = "call_vs_branch")]
#[cfg(target_arch = "x86_64")]
pub struct X86_64AsmCall;

#[cfg(target_arch = "x86_64")]
impl CallVsBranchVariant for X86_64AsmCall {
    fn name() -> &'static str {
        "x86_64-asm-call"
    }
    fn compute(value: u32) -> u32 {
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
}

#[variant(group = "call_vs_branch")]
#[cfg(target_arch = "x86_64")]
pub struct X86_64AsmBranch;

#[cfg(target_arch = "x86_64")]
impl CallVsBranchVariant for X86_64AsmBranch {
    fn name() -> &'static str {
        "x86_64-asm-branch"
    }
    fn compute(value: u32) -> u32 {
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
}

#[variant(group = "call_vs_branch")]
#[cfg(target_arch = "x86_64")]
pub struct X86_64AsmInline;

#[cfg(target_arch = "x86_64")]
impl CallVsBranchVariant for X86_64AsmInline {
    fn name() -> &'static str {
        "x86_64-asm-inline"
    }
    fn compute(value: u32) -> u32 {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expected_result(value: u32) -> u32 {
        let step1 = value.wrapping_mul(2); // double
        let step2 = step1.wrapping_add(10); // add_ten
        step2.wrapping_mul(step2) // square
    }

    #[test]
    fn test_all_match() {
        for v in 0..1000 {
            let expected = expected_result(v);
            assert_eq!(
                X86_64AsmCall::compute(v),
                expected,
                "X86_64AsmCall mismatch for {}",
                v
            );
            assert_eq!(
                X86_64AsmBranch::compute(v),
                expected,
                "X86_64AsmBranch mismatch for {}",
                v
            );
            assert_eq!(
                X86_64AsmInline::compute(v),
                expected,
                "X86_64AsmInline mismatch for {}",
                v
            );
        }
    }
}
