use crate::math::log2::Log2Variant;

#[crate::variant(group = "log2")]
pub struct FastapproxFast;

impl Log2Variant for FastapproxFast {
    #[inline]
    fn compute(a: f32) -> f32 {
        fastapprox::fast::log2(a)
    }
}

#[crate::variant(group = "log2")]
pub struct FastapproxFaster;

impl Log2Variant for FastapproxFaster {
    #[inline]
    fn compute(a: f32) -> f32 {
        fastapprox::faster::log2(a)
    }
}
