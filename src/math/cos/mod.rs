//! Cosine of f32 angles in radians.
#[cfg(test)]
mod test;
#[crate::algorithm(group = "cos", reference = "code::variants::Std")]
pub trait CosVariant: Send + 'static {
    /// Prepare shared state outside benchmark timing; stateless variants do nothing.
    fn prepare() {}
    fn compute(a: f32) -> f32;
}
crate::algorithm_modules!("math/cos");
