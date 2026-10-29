//! FFI bindings for C implementations of Xoroshiro128++.

#[cfg(c_implementation_active)]
mod ffi {
    extern "C" {
        pub fn xoroshiro128plusplus_c(s0: *mut u64, s1: *mut u64) -> u64;
    }
}

/// C implementation wrapper
#[cfg(c_implementation_active)]
pub fn xoroshiro_c_wrapper(s0: &mut u64, s1: &mut u64) -> u64 {
    unsafe { ffi::xoroshiro128plusplus_c(s0, s1) }
}

/// Check if C implementations are available
#[cfg(c_implementation_active)]
pub const C_IMPL_AVAILABLE: bool = true;

#[cfg(not(c_implementation_active))]
pub const C_IMPL_AVAILABLE: bool = false;

/// Name of the C compiler used
#[cfg(c_implementation_active)]
pub const COMPILER_NAME: Option<&str> = Some(env!("C_COMPILER_NAME"));

#[cfg(not(c_implementation_active))]
pub const COMPILER_NAME: Option<&str> = None;

// Stubs for missing C compiler
#[cfg(not(c_implementation_active))]
pub fn xoroshiro_c_wrapper(_s0: &mut u64, _s1: &mut u64) -> u64 {
    panic!("C implementation not compiled (requires GCC/Clang)")
}

#[cfg(c_implementation_active)]
use crate::random::xoroshiro::XoroshiroVariant;
#[cfg(c_implementation_active)]
use crate::variant;

#[cfg(c_implementation_active)]
#[variant(group = "xoroshiro")]
pub struct COriginal;

#[cfg(c_implementation_active)]
impl XoroshiroVariant for COriginal {
    fn name() -> &'static str {
        "c-original"
    }
    fn compute(s0: &mut u64, s1: &mut u64) -> u64 {
        xoroshiro_c_wrapper(s0, s1)
    }
}
