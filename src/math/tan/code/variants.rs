use crate::math::tan::TanVariant;

#[crate::variant(group = "tan")]
pub struct Std;
impl TanVariant for Std {
    #[inline]
    fn compute(a: f32) -> f32 {
        a.tan()
    }
}

#[crate::variant(group = "tan")]
pub struct Libm;
impl TanVariant for Libm {
    #[inline]
    fn compute(a: f32) -> f32 {
        libm::tanf(a)
    }
}

#[crate::variant(group = "tan")]
pub struct Micromath;
impl TanVariant for Micromath {
    #[inline]
    fn compute(a: f32) -> f32 {
        micromath::F32Ext::tan(a)
    }
}

#[crate::variant(group = "tan")]
pub struct FastapproxFast;
impl TanVariant for FastapproxFast {
    #[inline]
    fn compute(a: f32) -> f32 {
        fastapprox::fast::tan(a)
    }
}

#[crate::variant(group = "tan")]
pub struct FastapproxFaster;
impl TanVariant for FastapproxFaster {
    #[inline]
    fn compute(a: f32) -> f32 {
        fastapprox::faster::tan(a)
    }
}
