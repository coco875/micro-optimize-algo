use crate::math::pow2::Pow2Variant;

#[crate::variant(group = "pow2")]
pub struct Std;
impl Pow2Variant for Std {
    #[inline]
    fn compute(a: f32) -> f32 {
        a.exp2()
    }
}

#[crate::variant(group = "pow2")]
pub struct Libm;
impl Pow2Variant for Libm {
    #[inline]
    fn compute(a: f32) -> f32 {
        libm::exp2f(a)
    }
}

#[crate::variant(group = "pow2")]
pub struct Micromath;
impl Pow2Variant for Micromath {
    #[inline]
    fn compute(a: f32) -> f32 {
        micromath::F32Ext::powf(2.0, a)
    }
}

#[crate::variant(group = "pow2")]
pub struct FastapproxFast;
impl Pow2Variant for FastapproxFast {
    #[inline]
    fn compute(a: f32) -> f32 {
        fastapprox::fast::pow2(a)
    }
}

#[crate::variant(group = "pow2")]
pub struct FastapproxFaster;
impl Pow2Variant for FastapproxFaster {
    #[inline]
    fn compute(a: f32) -> f32 {
        fastapprox::faster::pow2(a)
    }
}
