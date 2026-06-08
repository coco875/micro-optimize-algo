//! Tests for call vs branch implementations

#[cfg(test)]
mod tests {
    use crate::control_flow::call_vs_branch::code::original::{Original, RustInline};
    use crate::control_flow::call_vs_branch::CallVsBranchVariant;

    fn verify_val(v: u32, expected: u32) {
        assert_eq!(Original::compute(v), expected);
        assert_eq!(RustInline::compute(v), expected);

        #[cfg(target_arch = "x86_64")]
        {
            use crate::control_flow::call_vs_branch::code::x86_64_asm::{
                X86_64AsmBranch, X86_64AsmCall, X86_64AsmInline,
            };
            assert_eq!(X86_64AsmCall::compute(v), expected);
            assert_eq!(X86_64AsmBranch::compute(v), expected);
            assert_eq!(X86_64AsmInline::compute(v), expected);
        }
    }

    #[test]
    fn test_all_variants() {
        let test_values: Vec<u32> = vec![0, 1, 2, 5, 10, 50, 100, 255, 500, 1000, 10000];
        for &v in &test_values {
            let step1 = v.wrapping_mul(2);
            let step2 = step1.wrapping_add(10);
            let expected = step2.wrapping_mul(step2);
            verify_val(v, expected);
        }
    }

    #[test]
    fn test_expected_computation() {
        // For x = 5: double(5) = 10, add_ten(10) = 20, square(20) = 400
        verify_val(5, 400);
    }

    #[test]
    fn test_zero() {
        // For x = 0: double(0) = 0, add_ten(0) = 10, square(10) = 100
        verify_val(0, 100);
    }
}
