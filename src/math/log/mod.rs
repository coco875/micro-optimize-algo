//! Natural logarithm ln(x), with base e.
#[cfg(test)]
mod test;
#[crate::algorithm(group = "log", reference = "code::variants::Std")]
pub trait LogVariant: Send + 'static {
    /// Prepare shared state outside benchmark timing; stateless variants do nothing.
    fn prepare() {}
    fn compute(a: f32) -> f32;
}
crate::algorithm_modules!("math/log");
