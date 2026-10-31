use crate::math::log2::Log2Variant;

#[crate::variant(group = "log2")]
pub struct Libm;

impl Log2Variant for Libm {
    #[inline]
    fn compute(a: f32) -> f32 {
        libm::log2f(a)
    }
}
