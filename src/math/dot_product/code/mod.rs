//! Dot product implementations.
//!
//! This module contains all implementation variants of the dot product algorithm.

pub use c_impl::C_IMPL_AVAILABLE;
pub use original::dot_product_original;
pub use scalar_opt::dot_product_scalar_opt;
#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub use x86_64_avx2::dot_product_x86_64_avx2;
#[cfg(target_arch = "x86_64")]
pub use x86_64_sse2::dot_product_x86_64_sse2;

crate::implementation_modules!("math/dot_product");
