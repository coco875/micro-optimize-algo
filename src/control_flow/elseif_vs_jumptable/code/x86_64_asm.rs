//! x86_64 Assembly implementations comparing:
//! - Else-if chain (Branch): Sequential comparisons with Jcc, O(n) worst case
//! - Jump table: Indexed dispatch, O(1) constant time  
//! - Branchless (CMOV): Conditional moves, no branch prediction needed
//!
//! # Else-If Chain Assembly (Branch instructions)
//! ```asm
//! dispatch_elseif:
//!     cmp dil, 0
//!     je .op0           ; Conditional BRANCH (Jcc)
//!     cmp dil, 1
//!     je .op1           ; Another branch
//!     cmp dil, 2        
//!     je .op2           ; Yet another branch
//!     ; ... repeat for each case
//!     xor eax, eax      ; default: return 0
//!     ret
//! ```
//!
//! # Jump Table Assembly
//! ```asm
//! dispatch_jumptable:
//!     cmp dil, 7
//!     ja .invalid           ; Bounds check
//!     movzx eax, dil
//!     mov eax, [table + rax*4]  ; Load multiplier
//!     imul eax, esi             ; Multiply
//!     ret
//! ```
//!
//! # Branchless Assembly (CMOV)
//! ```asm
//! dispatch_branchless:
//!     cmp dil, 7
//!     mov eax, 0            ; Default for invalid
//!     ja .done              ; Only one branch for bounds
//!     ; Use CMOV to select multiplier without branching  
//!     mov ecx, 1
//!     cmp dil, 0
//!     cmove eax, ecx        ; If opcode==0, mult=1
//!     mov ecx, 2
//!     cmp dil, 1  
//!     cmove eax, ecx        ; If opcode==1, mult=2
//!     ; ... etc
//! ```
//!
//! # Trade-offs
//!
//! | Aspect | Branch (Jcc) | Jump Table | Branchless (CMOV) |
//! |--------|--------------|------------|-------------------|
//! | Time complexity | O(n) | O(1) | O(n) but consistent |
//! | Branch misprediction | Yes, costly | One indirect | No |
//! | Best case | First case | All equal | All equal |
//! | Worst case | Last case | All equal | All equal |
//! | Random data perf | Poor | Good | Good |

use std::arch::asm;

/// Dispatch using chained if-else with conditional BRANCHES (Jcc instructions)
///
/// Each case requires a comparison and conditional jump (JE, JNE, etc.).
/// Subject to branch prediction - mispredictions cost 10-20 cycles.
/// Earlier cases are faster, later cases are slower.
#[inline(never)]
pub fn dispatch_branch(opcode: u8, value: u32) -> u32 {
    let result: u32;
    // Zero-extend opcode to u32 before passing to asm
    let opcode_ext = opcode as u32;
    
    unsafe {
        asm!(
            // Bounds check first
            "cmp {opcode:e}, 7",
            "ja 90f",                    // BRANCH: If > 7, jump to invalid handler
            
            // Case 0: identity - BRANCH
            "test {opcode:e}, {opcode:e}",
            "jnz 20f",                   // BRANCH: Jump if not zero
            "mov {result:e}, {value:e}",
            "jmp 99f",                   // Unconditional jump
            
            // Check remaining cases with BRANCHES
            "20:",
            "cmp {opcode:e}, 2",
            "je 30f",                    // BRANCH
            "cmp {opcode:e}, 3",
            "je 40f",                    // BRANCH
            "cmp {opcode:e}, 4",
            "je 50f",                    // BRANCH
            "cmp {opcode:e}, 5",
            "je 60f",                    // BRANCH
            "cmp {opcode:e}, 6",
            "je 70f",                    // BRANCH
            "cmp {opcode:e}, 7",
            "je 80f",                    // BRANCH
            // Must be case 1 (already checked 0, 2-7 above)
            "lea {result:e}, [{value:e} + {value:e}]",  // ×2
            "jmp 99f",
            
            // Case 2: ×3
            "30:",
            "lea {result:e}, [{value:e} + {value:e}*2]",
            "jmp 99f",
            
            // Case 3: ×4
            "40:",
            "shl {value:e}, 2",
            "mov {result:e}, {value:e}",
            "jmp 99f",
            
            // Case 4: ×5
            "50:",
            "lea {result:e}, [{value:e} + {value:e}*4]",
            "jmp 99f",
            
            // Case 5: ×6
            "60:",
            "lea {result:e}, [{value:e} + {value:e}*2]",
            "add {result:e}, {result:e}",
            "jmp 99f",
            
            // Case 6: ×7
            "70:",
            "mov {result:e}, {value:e}",
            "shl {value:e}, 3",
            "sub {value:e}, {result:e}",
            "mov {result:e}, {value:e}",
            "jmp 99f",
            
            // Case 7: ×8
            "80:",
            "shl {value:e}, 3",
            "mov {result:e}, {value:e}",
            "jmp 99f",
            
            // Invalid opcode
            "90:",
            "xor {result:e}, {result:e}",
            
            "99:",
            
            opcode = in(reg) opcode_ext,
            value = in(reg) value,
            result = out(reg) result,
            options(nostack, nomem),
        );
    }
    
    result
}

/// Dispatch using a TRUE JUMP TABLE with label offsets
///
/// Uses an array of relative offsets to code labels.
/// The CPU computes the target address and does an indirect jump.
/// O(1) time - same performance regardless of opcode value.
///
/// Assembly pattern:
/// ```asm
///     lea rax, [rip + jump_table]   ; Load base address
///     movsxd rcx, [rax + rdi*4]     ; Load 32-bit offset from table
///     add rax, rcx                   ; Compute absolute target
///     jmp rax                        ; Indirect jump to case handler
///
/// jump_table:
///     .long .case_0 - jump_table
///     .long .case_1 - jump_table
///     ; ...
///
/// .case_0:
///     mov eax, esi                   ; result = value * 1
///     jmp .done
/// .case_1:
///     lea eax, [esi + esi]           ; result = value * 2
///     jmp .done
/// ; ...
/// ```
#[inline(never)]
pub fn dispatch_jumptable(opcode: u8, value: u32) -> u32 {
    let result: u32;
    let opcode_ext = opcode as u32;
    
    unsafe {
        asm!(
            // Bounds check
            "cmp {opcode:e}, 7",
            "ja 92f",                      // If > 7, jump to invalid handler
            
            // === JUMP TABLE DISPATCH ===
            // Load base address of jump table (RIP-relative)
            "lea {base}, [rip + 500f]",
            
            // Load 32-bit signed offset from table: offset = table[opcode]
            "movsxd {offset}, dword ptr [{base} + {opcode:r}*4]",
            
            // Compute absolute target address: target = base + offset
            "add {base}, {offset}",
            
            // Indirect jump to computed address
            "jmp {base}",
            
            // === JUMP TABLE DATA (8 entries × 4 bytes = 32 bytes) ===
            // Each entry is the offset from the table base to the case handler
            ".p2align 2",                   // Align to 4 bytes
            "500:",                         // jump_table label
            ".long 600f - 500b",            // case 0: offset to label 600
            ".long 602f - 500b",            // case 1: offset to label 602
            ".long 604f - 500b",            // case 2: offset to label 604
            ".long 606f - 500b",            // case 3: offset to label 606
            ".long 608f - 500b",            // case 4: offset to label 608
            ".long 620f - 500b",            // case 5: offset to label 620
            ".long 622f - 500b",            // case 6: offset to label 622
            ".long 624f - 500b",            // case 7: offset to label 624
            
            // === CASE HANDLERS ===
            // Case 0: result = value * 1 (identity)
            "600:",
            "mov {result:e}, {value:e}",
            "jmp 99f",
            
            // Case 1: result = value * 2
            "602:",
            "lea {result:e}, [{value:e} + {value:e}]",
            "jmp 99f",
            
            // Case 2: result = value * 3
            "604:",
            "lea {result:e}, [{value:e} + {value:e}*2]",
            "jmp 99f",
            
            // Case 3: result = value * 4
            "606:",
            "mov {result:e}, {value:e}",
            "shl {result:e}, 2",
            "jmp 99f",
            
            // Case 4: result = value * 5
            "608:",
            "lea {result:e}, [{value:e} + {value:e}*4]",
            "jmp 99f",
            
            // Case 5: result = value * 6
            "620:",
            "lea {result:e}, [{value:e} + {value:e}*2]",
            "add {result:e}, {result:e}",
            "jmp 99f",
            
            // Case 6: result = value * 7 (8 - 1)
            "622:",
            "mov {result:e}, {value:e}",
            "shl {value:e}, 3",
            "sub {value:e}, {result:e}",
            "mov {result:e}, {value:e}",
            "jmp 99f",
            
            // Case 7: result = value * 8
            "624:",
            "mov {result:e}, {value:e}",
            "shl {result:e}, 3",
            "jmp 99f",
            
            // Invalid opcode handler
            "92:",
            "xor {result:e}, {result:e}",
            
            // Done
            "99:",
            
            opcode = in(reg) opcode_ext,
            value = in(reg) value,
            result = out(reg) result,
            base = out(reg) _,
            offset = out(reg) _,
            options(nostack, nomem),
        );
    }
    
    result
}

/// Dispatch using CMOV - completely branchless (no branches at all)
///
/// Uses conditional moves to select the result without any branches.
/// No branch misprediction possible - consistent performance.
/// Evaluates ALL instructions but no pipeline stalls.
#[inline(never)]
pub fn dispatch_branchless(opcode: u8, value: u32) -> u32 {
    // Use lookup table approach for branchless - simplest and fastest
    static MULTIPLIERS: [u32; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    
    let result: u32;
    let opcode_ext = opcode as u32;
    
    unsafe {
        asm!(
            // First, clamp opcode to valid range (0-7) to avoid out-of-bounds read
            // If opcode > 7, use 0 as index (we'll zero the result later anyway)
            "mov {idx:e}, {opcode:e}",
            "cmp {opcode:e}, 7",
            "mov {tmp:e}, 0",
            "cmova {idx:e}, {tmp:e}",   // If opcode > 7, idx = 0 (branchless!)
            
            // Load multiplier from table using clamped index
            "mov {mult:e}, [{table} + {idx:r}*4]",
            
            // Compute result = value * multiplier
            "imul {result:e}, {value:e}, 1",
            "imul {result:e}, {mult:e}",
            
            // Redo the comparison since IMUL clobbered the flags
            "cmp {opcode:e}, 7",
            
            // If opcode was invalid (> 7), zero the result
            // cmova = conditional move if above (CF=0 and ZF=0)
            "mov {tmp:e}, 0",
            "cmova {result:e}, {tmp:e}",  // If opcode > 7, result = 0 (branchless!)
            
            opcode = in(reg) opcode_ext,
            value = in(reg) value,
            table = in(reg) MULTIPLIERS.as_ptr(),
            result = out(reg) result,
            mult = out(reg) _,
            tmp = out(reg) _,
            idx = out(reg) _,
            options(nostack, readonly),
        );
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dispatch(dispatch_fn: fn(u8, u32) -> u32, name: &str) {
        assert_eq!(dispatch_fn(0, 10), 10, "{}: op 0: identity", name);
        assert_eq!(dispatch_fn(1, 10), 20, "{}: op 1: ×2", name);
        assert_eq!(dispatch_fn(2, 10), 30, "{}: op 2: ×3", name);
        assert_eq!(dispatch_fn(3, 10), 40, "{}: op 3: ×4", name);
        assert_eq!(dispatch_fn(4, 10), 50, "{}: op 4: ×5", name);
        assert_eq!(dispatch_fn(5, 10), 60, "{}: op 5: ×6", name);
        assert_eq!(dispatch_fn(6, 10), 70, "{}: op 6: ×7", name);
        assert_eq!(dispatch_fn(7, 10), 80, "{}: op 7: ×8", name);
        assert_eq!(dispatch_fn(8, 10), 0, "{}: invalid opcode 8", name);
        assert_eq!(dispatch_fn(255, 10), 0, "{}: invalid opcode 255", name);
    }

    #[test]
    fn test_dispatch_branch() {
        test_dispatch(dispatch_branch, "branch");
    }

    #[test]
    fn test_dispatch_jumptable() {
        test_dispatch(dispatch_jumptable, "jumptable");
    }

    #[test]
    fn test_dispatch_branchless() {
        test_dispatch(dispatch_branchless, "branchless");
    }
}
