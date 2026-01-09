//! Implementation variants for branch vs jumptable vs branchless comparison

pub mod c_impl;
pub mod original;
#[cfg(target_arch = "x86_64")]
pub mod x86_64_asm;

/// Function signature: maps an opcode (0-7) to a multiplier
pub type DispatchFn = fn(u8, u32) -> u32;

use crate::utils::VariantInfo;

/// Returns all available variants
pub fn get_variants() -> Vec<VariantInfo<DispatchFn>> {
    #[allow(unused_mut)]
    let mut variants: Vec<VariantInfo<DispatchFn>> = vec![VariantInfo {
        name: "original",
        description: "Rust match expression (compiler-optimized)",
        function: original::dispatch_operation,
    }];

    #[cfg(target_arch = "x86_64")]
    {
        variants.push(VariantInfo {
            name: "x86_64-asm-branch",
            description: "x86_64 assembly with conditional branches (Jcc)",
            function: x86_64_asm::dispatch_branch,
        });
        variants.push(VariantInfo {
            name: "x86_64-asm-jumptable",
            description: "x86_64 assembly with indexed jump table lookup",
            function: x86_64_asm::dispatch_jumptable,
        });
        variants.push(VariantInfo {
            name: "x86_64-asm-branchless",
            description: "x86_64 assembly branchless with CMOV",
            function: x86_64_asm::dispatch_branchless,
        });
    }

    // Register C implementations if available
    if c_impl::C_IMPL_AVAILABLE {
        variants.push(VariantInfo {
            name: "c-elseif",
            description: "C if-else if chain",
            function: c_impl::dispatch_operation_c_elseif,
        });

        variants.push(VariantInfo {
            name: "c-switch",
            description: "C switch statement (likely jumptable)",
            function: c_impl::dispatch_operation_c_switch,
        });
    }

    variants
}
