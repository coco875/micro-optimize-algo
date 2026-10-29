//! call vs branch implementations and independent workloads.
#[cfg(test)]
pub mod test;
pub use code::*;

#[crate::algorithm(group = "call_vs_branch", reference = "code::original::Original")]
pub trait CallVsBranchVariant: Send + 'static {
    /// Variant name
    fn name() -> &'static str;
    /// Computation function
    fn compute(val: u32) -> u32;
}

crate::algorithm_modules!("control_flow/call_vs_branch");
