//! Floating-point powers a raised to b; comparison fixtures use positive bases.
#[cfg(test)]
mod test;
#[crate::algorithm(group = "pow", reference = "code::variants::Std")]
pub trait PowVariant: Send + 'static {
    /// Prepare shared state outside benchmark timing; stateless variants do nothing.
    fn prepare() {}
    fn compute(a: f32, b: f32) -> f32;
}
crate::algorithm_modules!("math/pow");
