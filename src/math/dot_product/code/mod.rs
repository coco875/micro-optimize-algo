//! Dot product implementations.
//!
//! This module contains all implementation variants of the dot product algorithm.

mod original;
mod scalar_opt;
#[cfg(target_arch = "x86_64")]
mod x86_64_sse2;
pub mod c_impl;

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
mod x86_64_avx2;

pub use original::dot_product_original;
pub use scalar_opt::dot_product_scalar_opt;
#[cfg(target_arch = "x86_64")]
pub use x86_64_sse2::dot_product_x86_64_sse2;
pub use c_impl::{dot_product_c_original, dot_product_c_scalar_opt, C_IMPL_AVAILABLE};
#[cfg(target_arch = "x86_64")]
pub use c_impl::dot_product_c_x86_64_sse2;

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub use x86_64_avx2::dot_product_x86_64_avx2;

/// Trait for dot product implementations
pub trait DotProduct {
    /// Compute the dot product of two slices
    fn dot_product(a: &[f32], b: &[f32]) -> f32;
    
    /// Name of this implementation variant
    fn name() -> &'static str;
}

/// Implementation info for runtime variant selection
pub struct VariantInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub function: fn(&[f32], &[f32]) -> f32,
    pub compiler: Option<&'static str>,
}

/// Get all available variants for the current CPU
pub fn available_variants() -> Vec<VariantInfo> {
    let mut variants = vec![
        VariantInfo {
            name: "original",
            description: "Clean, idiomatic Rust reference implementation",
            function: dot_product_original,
            compiler: None,
        },
        VariantInfo {
            name: "scalar_opt",
            description: "Optimized scalar implementation (manual loop unrolling)",
            function: dot_product_scalar_opt,
            compiler: None,
        },
    ];

    #[cfg(target_arch = "x86_64")]
    {
        variants.push(VariantInfo {
            name: "x86_64-sse2",
            description: "x86_64 with SSE2 SIMD intrinsics",
            function: dot_product_x86_64_sse2,
            compiler: None,
        });
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    {
        variants.push(VariantInfo {
            name: "x86_64-avx2",
            description: "x86_64 with AVX2 SIMD intrinsics",
            function: dot_product_x86_64_avx2,
            compiler: None,
        });
    }

    // Add C implementations if available
    if C_IMPL_AVAILABLE {
        let compiler = env!("C_COMPILER_NAME");
        
        variants.push(VariantInfo {
            name: "c-original",
            description: "C reference implementation",
            function: dot_product_c_original,
            compiler: Some(compiler),
        });
        variants.push(VariantInfo {
            name: "c-scalar_opt",
            description: "C optimized scalar implementation",
            function: dot_product_c_scalar_opt,
            compiler: Some(compiler),
        });
        #[cfg(target_arch = "x86_64")]
        variants.push(VariantInfo {
            name: "c-x86_64-sse2",
            description: "C with SSE2 SIMD intrinsics",
            function: dot_product_c_x86_64_sse2,
            compiler: Some(compiler),
        });
    }

    variants
}

