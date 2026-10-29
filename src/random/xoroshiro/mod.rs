//! xoroshiro implementations and independent workloads.
#[cfg(test)]
pub mod test;
pub use code::*;

#[crate::algorithm(group = "xoroshiro", reference = "code::original::Original")]
pub trait XoroshiroVariant: Send + 'static {
    /// Variant name
    fn name() -> &'static str;
    /// Computation step function
    fn compute(s0: &mut u64, s1: &mut u64) -> u64;
}

crate::algorithm_modules!("random/xoroshiro");
