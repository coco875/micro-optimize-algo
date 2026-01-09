mod original;
#[cfg(target_arch = "x86_64")]
mod x86_64_asm;

pub use original::xoroshiro_original;
#[cfg(target_arch = "x86_64")]
pub use x86_64_asm::xoroshiro_x86_64_asm;

pub mod c_impl;

use crate::utils::VariantInfo;

pub fn available_variants() -> Vec<VariantInfo<fn(&mut u64, &mut u64) -> u64>> {
    let mut variants: Vec<VariantInfo<fn(&mut u64, &mut u64) -> u64>> = vec![VariantInfo {
        name: "original",
        function: original::xoroshiro_original,
        description: "Original pure Rust implementation",
    }];

    #[cfg(target_arch = "x86_64")]
    variants.push(VariantInfo {
        name: "x86_64-asm",
        function: x86_64_asm::xoroshiro_x86_64_asm,
        description: "Hand-written x86_64 assembly",
    });

    if c_impl::C_IMPL_AVAILABLE {
        variants.push(VariantInfo {
            name: "c-original",
            function: c_impl::xoroshiro_c_wrapper,
            description: "C implementation of Xoroshiro128++",
        });
    }

    variants
}
