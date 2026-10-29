//! elseif vs jumptable implementations and independent workloads.
#[cfg(test)]
pub mod test;
pub use code::*;

#[crate::algorithm(group = "elseif_vs_jumptable", reference = "code::original::Original")]
pub trait ElseIfVsJumpTableVariant: Send + 'static {
    /// Variant name
    fn name() -> &'static str;
    /// Computation function
    fn compute(op: u8, val: u32) -> u32;
}

crate::algorithm_modules!("control_flow/elseif_vs_jumptable");
