//! Implementation variants for branch vs jumptable vs branchless comparison

pub mod original;
pub mod c_impl;
#[cfg(target_arch = "x86_64")]
pub mod x86_64_asm;

/// Function signature: maps an opcode (0-7) to a multiplier
pub type DispatchFn = fn(u8, u32) -> u32;

/// Variant descriptor
pub struct Variant {
    pub name: &'static str,
    pub description: &'static str,
    pub func: DispatchFn,
}

/// Returns all available variants
pub fn get_variants() -> Vec<Variant> {
    #[allow(unused_mut)]
    let mut variants = vec![
        Variant {
            name: "original",
            description: "Rust match expression (compiler-optimized)",
            func: original::dispatch_operation,
        },
    ];
    
    #[cfg(target_arch = "x86_64")]
    {
        variants.push(Variant {
            name: "x86_64-asm-branch",
            description: "x86_64 assembly with conditional branches (Jcc)",
            func: x86_64_asm::dispatch_branch,
        });
        variants.push(Variant {
            name: "x86_64-asm-jumptable",
            description: "x86_64 assembly with indexed jump table lookup",
            func: x86_64_asm::dispatch_jumptable,
        });
        variants.push(Variant {
            name: "x86_64-asm-branchless",
            description: "x86_64 assembly branchless with CMOV",
            func: x86_64_asm::dispatch_branchless,
        });
    }

    // Register C implementations if available
    if c_impl::C_IMPL_AVAILABLE {
        variants.push(Variant {
            name: "c-elseif",
            description: "C if-else if chain",
            func: c_impl::dispatch_operation_c_elseif,
        });

        variants.push(Variant {
            name: "c-switch",
            description: "C switch statement (likely jumptable)",
            func: c_impl::dispatch_operation_c_switch,
        });
    }
    
    variants
}
