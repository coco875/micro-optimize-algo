//! Implementation variants for call vs branch comparison

pub mod original;
#[cfg(target_arch = "x86_64")]
pub mod x86_64_asm;
