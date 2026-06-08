//! Implementation variants for branch vs jumptable vs branchless comparison

pub mod c_impl;
pub mod original;
#[cfg(target_arch = "x86_64")]
pub mod x86_64_asm;
