//! Base-two exponential: pow2(x) means 2 raised to x, not x squared.
#[cfg(test)]
mod test;
#[crate::algorithm(group = "pow2", reference = "code::variants::Std")]
pub trait Pow2Variant: Send + 'static {
    /// Prepare shared state outside benchmark timing; stateless variants do nothing.
    fn prepare() {}
    fn compute(a: f32) -> f32;
}
crate::algorithm_modules!("math/pow2");
