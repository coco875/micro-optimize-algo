//! Implementation variants for call vs branch comparison

pub mod original;
pub mod x86_64_asm;

/// Function signature for the test functions
pub type TestFn = fn(u32) -> u32;

/// Variant descriptor
pub struct Variant {
    pub name: &'static str,
    pub description: &'static str,
    pub func: TestFn,
}

/// Returns all available variants
pub fn get_variants() -> Vec<Variant> {
    vec![
        Variant {
            name: "original",
            description: "Rust function calls (compiler decides inlining)",
            func: original::process_with_calls,
        },
        Variant {
            name: "x86_64-asm-call",
            description: "x86_64 assembly with explicit CALL/RET",
            func: x86_64_asm::process_with_calls,
        },
        Variant {
            name: "x86_64-asm-branch",
            description: "x86_64 assembly with JMP branches (no CALL overhead)",
            func: x86_64_asm::process_with_branch,
        },
        Variant {
            name: "x86_64-asm-inline",
            description: "x86_64 assembly fully inlined (no jumps)",
            func: x86_64_asm::process_inline,
        },
    ]
}
