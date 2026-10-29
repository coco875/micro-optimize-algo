//! grad dot implementations and independent workloads.
#[cfg(test)]
pub mod test;
pub use code::*;

#[crate::algorithm(group = "grad_dot", reference = "code::Original")]
pub trait GradDotVariant: Send + 'static {
    /// Variant name
    fn name() -> &'static str;
    /// Computation function
    fn compute(hashes: &[usize], x: &[f64], y: &[f64], z: &[f64]) -> f64;
}

crate::algorithm_modules!("math/grad_dot");
