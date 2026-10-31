//! Sine of f32 angles in radians.
#[cfg(test)]
mod test;
#[crate::algorithm(group = "sin", reference = "code::variants::Std")]
pub trait SinVariant: Send + 'static {
    /// Prepare shared state outside benchmark timing; stateless variants do nothing.
    fn prepare() {}
    fn compute(a: f32) -> f32;
}
crate::algorithm_modules!("math/sin");
