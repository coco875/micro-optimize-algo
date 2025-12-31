mod original;
mod x86_64_asm;

pub use original::xoroshiro_original;
pub use x86_64_asm::xoroshiro_x86_64_asm;

#[cfg(c_implementation_active)]
pub mod c_impl;

pub struct VariantInfo {
    pub name: &'static str,
    pub function: fn(&mut u64, &mut u64) -> u64,
    pub description: &'static str,
    pub compiler: Option<&'static str>,
}

pub fn available_variants() -> Vec<VariantInfo> {
    let mut variants = vec![
        VariantInfo {
            name: "original",
            function: original::xoroshiro_original,
            description: "Original pure Rust implementation",
            compiler: None,
        },
        VariantInfo {
            name: "x86_64-asm",
            function: x86_64_asm::xoroshiro_x86_64_asm,
            description: "Hand-written x86_64 assembly",
            compiler: None,
        },
    ];

    #[cfg(c_implementation_active)]
    variants.push(c_impl::VARIANT);

    variants
}
