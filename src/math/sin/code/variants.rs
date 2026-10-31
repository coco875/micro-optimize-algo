use crate::math::sin::SinVariant;

#[crate::variant(group = "sin")]
pub struct Std;
impl SinVariant for Std {
    #[inline]
    fn compute(a: f32) -> f32 {
        a.sin()
    }
}

#[crate::variant(group = "sin")]
pub struct Libm;
impl SinVariant for Libm {
    #[inline]
    fn compute(a: f32) -> f32 {
        libm::sinf(a)
    }
}

#[crate::variant(group = "sin")]
pub struct Micromath;
impl SinVariant for Micromath {
    #[inline]
    fn compute(a: f32) -> f32 {
        micromath::F32Ext::sin(a)
    }
}

#[crate::variant(group = "sin")]
pub struct FastapproxFast;
impl SinVariant for FastapproxFast {
    #[inline]
    fn compute(a: f32) -> f32 {
        fastapprox::fast::sinfull(a)
    }
}

#[crate::variant(group = "sin")]
pub struct FastapproxFaster;
impl SinVariant for FastapproxFaster {
    #[inline]
    fn compute(a: f32) -> f32 {
        fastapprox::faster::sinfull(a)
    }
}
