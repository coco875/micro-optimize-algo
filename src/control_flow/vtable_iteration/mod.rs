//! vtable iteration implementations and independent workloads.
#[cfg(test)]
pub mod test;
pub use code::*;

#[crate::algorithm(
    group = "vtable_iteration",
    reference = "code::variants::VTableContiguous"
)]
pub trait VTableIterationVariant: Send + 'static {
    /// Variant name
    fn name() -> &'static str;
    /// Creator function for the object pool
    fn create(objects: &[(u8, u8)]) -> Box<dyn ObjectPool + Send>;
}

crate::algorithm_modules!("control_flow/vtable_iteration");
