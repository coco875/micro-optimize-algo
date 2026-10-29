pub use original::xoroshiro_original;
#[cfg(target_arch = "x86_64")]
pub use x86_64_asm::xoroshiro_x86_64_asm;

crate::implementation_modules!("random/xoroshiro");
