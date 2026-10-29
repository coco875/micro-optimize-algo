//! dot product implementations and independent workloads.
#[cfg(test)]
pub mod test;
pub use code::*;

#[crate::algorithm(group = "dot_product", reference = "code::original::Original")]
pub trait DotProductVariant: Send + 'static {
    /// Variant name
    fn name() -> &'static str;
    /// Computation function
    fn compute(a: &[f32], b: &[f32]) -> f32;
}

crate::algorithm_modules!("math/dot_product");
