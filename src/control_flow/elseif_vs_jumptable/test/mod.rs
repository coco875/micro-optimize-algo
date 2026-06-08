//! Tests for else-if vs jump table implementations

#[cfg(test)]
mod tests {
    use crate::control_flow::elseif_vs_jumptable::code::original::Original;
    use crate::control_flow::elseif_vs_jumptable::ElseIfVsJumpTableVariant;

    fn verify_val(opcode: u8, value: u32, expected: u32) {
        assert_eq!(Original::compute(opcode, value), expected);

        #[cfg(target_arch = "x86_64")]
        {
            use crate::control_flow::elseif_vs_jumptable::code::x86_64_asm::{
                X86_64AsmBranch, X86_64AsmBranchless, X86_64AsmJumpTable,
            };
            assert_eq!(X86_64AsmBranch::compute(opcode, value), expected);
            assert_eq!(X86_64AsmJumpTable::compute(opcode, value), expected);
            assert_eq!(X86_64AsmBranchless::compute(opcode, value), expected);
        }

        #[cfg(c_implementation_active)]
        {
            use crate::control_flow::elseif_vs_jumptable::code::c_impl::{CElseIf, CSwitch};
            assert_eq!(CElseIf::compute(opcode, value), expected);
            assert_eq!(CSwitch::compute(opcode, value), expected);
        }
    }

    #[test]
    fn test_all_opcodes() {
        let test_cases: Vec<(u8, u32, u32)> = vec![
            (0, 10, 10),
            (1, 10, 20),
            (2, 10, 30),
            (3, 10, 40),
            (4, 10, 50),
            (5, 10, 60),
            (6, 10, 70),
            (7, 10, 80),
            (8, 10, 0),
            (255, 10, 0),
        ];
        for &(opcode, value, expected) in &test_cases {
            verify_val(opcode, value, expected);
        }
    }

    #[test]
    fn test_edge_values() {
        // Test with 0
        verify_val(0, 0, 0);
        // Test with 1
        verify_val(0, 1, 1);
        // Test with large value
        verify_val(1, 1000000, 2000000);
    }
}
