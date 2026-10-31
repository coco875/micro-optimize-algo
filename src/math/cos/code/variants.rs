use crate::math::cos::CosVariant;

#[crate::variant(group = "cos")]
pub struct Std;
impl CosVariant for Std {
    #[inline]
    fn compute(a: f32) -> f32 {
        a.cos()
    }
}

#[crate::variant(group = "cos")]
pub struct Libm;
impl CosVariant for Libm {
    #[inline]
    fn compute(a: f32) -> f32 {
        libm::cosf(a)
    }
}

#[crate::variant(group = "cos")]
pub struct Micromath;
impl CosVariant for Micromath {
    #[inline]
    fn compute(a: f32) -> f32 {
        micromath::F32Ext::cos(a)
    }
}

#[crate::variant(group = "cos")]
pub struct FastapproxFast;
impl CosVariant for FastapproxFast {
    #[inline]
    fn compute(a: f32) -> f32 {
        fastapprox::fast::cosfull(a)
    }
}

#[crate::variant(group = "cos")]
pub struct FastapproxFaster;
impl CosVariant for FastapproxFaster {
    #[inline]
    fn compute(a: f32) -> f32 {
        fastapprox::faster::cosfull(a)
    }
}
