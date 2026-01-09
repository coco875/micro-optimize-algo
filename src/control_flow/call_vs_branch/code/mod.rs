//! Implementation variants for call vs branch comparison

pub mod original;
#[cfg(target_arch = "x86_64")]
pub mod x86_64_asm;

/// Function signature for the test functions
pub type TestFn = fn(u32) -> u32;

use crate::utils::VariantInfo;

/// Returns all available variants
pub fn get_variants() -> Vec<VariantInfo<TestFn>> {
    #[allow(unused_mut)]
    let mut variants: Vec<VariantInfo<TestFn>> = vec![VariantInfo {
        name: "original",
        description: "Rust function calls (compiler decides inlining)",
        function: original::process_with_calls,
    }];

    #[cfg(target_arch = "x86_64")]
    {
        variants.push(VariantInfo {
            name: "x86_64-asm-call",
            description: "x86_64 assembly with explicit CALL/RET",
            function: x86_64_asm::process_with_calls,
        });
        variants.push(VariantInfo {
            name: "x86_64-asm-branch",
            description: "x86_64 assembly with JMP branches (no CALL overhead)",
            function: x86_64_asm::process_with_branch,
        });
        variants.push(VariantInfo {
            name: "x86_64-asm-inline",
            description: "x86_64 assembly fully inlined (no jumps)",
            function: x86_64_asm::process_inline,
        });
    }

    variants
}
