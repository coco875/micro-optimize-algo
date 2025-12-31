//! FFI bindings for C implementations.

#[cfg(c_implementation_active)]
mod ffi {
    use std::os::raw::c_float;
    use libc::size_t;
    
    extern "C" {
        pub fn dot_product_c_original(a: *const c_float, b: *const c_float, len: size_t) -> c_float;
        pub fn dot_product_c_scalar_opt(a: *const c_float, b: *const c_float, len: size_t) -> c_float;
        pub fn dot_product_c_x86_64_sse2(a: *const c_float, b: *const c_float, len: size_t) -> c_float;
    }
}

/// C original implementation wrapper
#[cfg(c_implementation_active)]
pub fn dot_product_c_original(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");
    unsafe {
        ffi::dot_product_c_original(a.as_ptr(), b.as_ptr(), a.len())
    }
}

/// C scalar_opt implementation wrapper
#[cfg(c_implementation_active)]
pub fn dot_product_c_scalar_opt(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");
    unsafe {
        ffi::dot_product_c_scalar_opt(a.as_ptr(), b.as_ptr(), a.len())
    }
}

/// C x86_64 SSE2 implementation wrapper
#[cfg(c_implementation_active)]
pub fn dot_product_c_x86_64_sse2(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");
    unsafe {
        ffi::dot_product_c_x86_64_sse2(a.as_ptr(), b.as_ptr(), a.len())
    }
}

/// Check if C implementations are available
#[cfg(c_implementation_active)]
pub const C_IMPL_AVAILABLE: bool = true;

#[cfg(not(c_implementation_active))]
pub const C_IMPL_AVAILABLE: bool = false;

// Stub implementations for missing C compiler
#[cfg(not(c_implementation_active))]
pub fn dot_product_c_original(_a: &[f32], _b: &[f32]) -> f32 {
    panic!("C implementation not compiled (requires GCC/Clang)")
}

#[cfg(not(c_implementation_active))]
pub fn dot_product_c_scalar_opt(_a: &[f32], _b: &[f32]) -> f32 {
    panic!("C implementation not compiled (requires GCC/Clang)")
}

#[cfg(not(c_implementation_active))]
pub fn dot_product_c_x86_64_sse2(_a: &[f32], _b: &[f32]) -> f32 {
    panic!("C implementation not compiled (requires GCC/Clang)")
}
