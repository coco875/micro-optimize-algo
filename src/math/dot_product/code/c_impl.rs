//! FFI bindings for C implementations.

#[cfg(c_implementation_active)]
mod ffi {
    use libc::size_t;
    use std::os::raw::c_float;

    extern "C" {
        pub fn dot_product_c_original(a: *const c_float, b: *const c_float, len: size_t)
            -> c_float;
        pub fn dot_product_c_scalar_opt(
            a: *const c_float,
            b: *const c_float,
            len: size_t,
        ) -> c_float;
        #[cfg(target_arch = "x86_64")]
        pub fn dot_product_c_x86_64_sse2(
            a: *const c_float,
            b: *const c_float,
            len: size_t,
        ) -> c_float;
    }
}

/// C original implementation wrapper
#[cfg(c_implementation_active)]
pub fn dot_product_c_original(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");
    unsafe { ffi::dot_product_c_original(a.as_ptr(), b.as_ptr(), a.len()) }
}

/// C scalar_opt implementation wrapper
#[cfg(c_implementation_active)]
pub fn dot_product_c_scalar_opt(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");
    unsafe { ffi::dot_product_c_scalar_opt(a.as_ptr(), b.as_ptr(), a.len()) }
}

/// C x86_64 SSE2 implementation wrapper
#[cfg(all(c_implementation_active, target_arch = "x86_64"))]
pub fn dot_product_c_x86_64_sse2(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");
    unsafe { ffi::dot_product_c_x86_64_sse2(a.as_ptr(), b.as_ptr(), a.len()) }
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

// Stub implementations for missing C compiler
#[cfg(not(c_implementation_active))]
pub fn dot_product_c_original(_a: &[f32], _b: &[f32]) -> f32 {
    panic!("C implementation not compiled (requires GCC/Clang)")
}

#[cfg(not(c_implementation_active))]
pub fn dot_product_c_scalar_opt(_a: &[f32], _b: &[f32]) -> f32 {
    panic!("C implementation not compiled (requires GCC/Clang)")
}

#[cfg(all(not(c_implementation_active), target_arch = "x86_64"))]
pub fn dot_product_c_x86_64_sse2(_a: &[f32], _b: &[f32]) -> f32 {
    panic!("C implementation not compiled (requires GCC/Clang)")
}

#[cfg(c_implementation_active)]
use crate::math::dot_product::DotProductVariant;
#[cfg(c_implementation_active)]
use crate::variant;

#[cfg(c_implementation_active)]
#[variant(group = "dot_product")]
pub struct COriginal;

#[cfg(c_implementation_active)]
impl DotProductVariant for COriginal {
    fn name() -> &'static str {
        "c-original"
    }
    fn compute(a: &[f32], b: &[f32]) -> f32 {
        dot_product_c_original(a, b)
    }
}

#[cfg(c_implementation_active)]
#[variant(group = "dot_product")]
pub struct CScalarOpt;

#[cfg(c_implementation_active)]
impl DotProductVariant for CScalarOpt {
    fn name() -> &'static str {
        "c-scalar_opt"
    }
    fn compute(a: &[f32], b: &[f32]) -> f32 {
        dot_product_c_scalar_opt(a, b)
    }
}

#[cfg(all(c_implementation_active, target_arch = "x86_64"))]
#[variant(group = "dot_product")]
pub struct CX86_64Sse2;

#[cfg(all(c_implementation_active, target_arch = "x86_64"))]
impl DotProductVariant for CX86_64Sse2 {
    fn name() -> &'static str {
        "c-x86_64-sse2"
    }
    fn compute(a: &[f32], b: &[f32]) -> f32 {
        dot_product_c_x86_64_sse2(a, b)
    }
}
