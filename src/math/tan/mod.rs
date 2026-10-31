//! Tangent of f32 angles in radians, away from its poles.
#[cfg(test)]
mod test;
#[crate::algorithm(group = "tan", reference = "code::variants::Std")]
pub trait TanVariant: Send + 'static {
    /// Prepare shared state outside benchmark timing; stateless variants do nothing.
    fn prepare() {}
    fn compute(a: f32) -> f32;
}
crate::algorithm_modules!("math/tan");
