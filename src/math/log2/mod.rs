//! Scalar f32 logarithms: bit, interpolation, and series approximations versus std.

#[cfg(test)]
mod test;

pub use code::bit_approx::log2;

#[crate::algorithm(group = "log2", reference = "code::std::Std")]
pub trait Log2Variant: Send + 'static {
    /// Prepare shared state outside benchmark timing; stateless variants do nothing.
    fn prepare() {}
    fn compute(a: f32) -> f32;
}

crate::algorithm_modules!("math/log2");
