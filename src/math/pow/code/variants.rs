use crate::math::pow::PowVariant;

#[crate::variant(group = "pow")]
pub struct Std;
impl PowVariant for Std {
    #[inline]
    fn compute(a: f32, b: f32) -> f32 {
        a.powf(b)
    }
}

#[crate::variant(group = "pow")]
pub struct Libm;
impl PowVariant for Libm {
    #[inline]
    fn compute(a: f32, b: f32) -> f32 {
        libm::powf(a, b)
    }
}

#[crate::variant(group = "pow")]
pub struct Micromath;
impl PowVariant for Micromath {
    #[inline]
    fn compute(a: f32, b: f32) -> f32 {
        micromath::F32Ext::powf(a, b)
    }
}

#[crate::variant(group = "pow")]
pub struct FastapproxFast;
impl PowVariant for FastapproxFast {
    #[inline]
    fn compute(a: f32, b: f32) -> f32 {
        fastapprox::fast::pow(a, b)
    }
}

#[crate::variant(group = "pow")]
pub struct FastapproxFaster;
impl PowVariant for FastapproxFaster {
    #[inline]
    fn compute(a: f32, b: f32) -> f32 {
        fastapprox::faster::pow(a, b)
    }
}
