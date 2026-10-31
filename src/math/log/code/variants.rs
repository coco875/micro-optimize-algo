use crate::math::log::LogVariant;

#[crate::variant(group = "log")]
pub struct Std;
impl LogVariant for Std {
    #[inline]
    fn compute(a: f32) -> f32 {
        a.ln()
    }
}

#[crate::variant(group = "log")]
pub struct Libm;
impl LogVariant for Libm {
    #[inline]
    fn compute(a: f32) -> f32 {
        libm::logf(a)
    }
}

#[crate::variant(group = "log")]
pub struct Micromath;
impl LogVariant for Micromath {
    #[inline]
    fn compute(a: f32) -> f32 {
        micromath::F32Ext::ln(a)
    }
}

#[crate::variant(group = "log")]
pub struct FastapproxFast;
impl LogVariant for FastapproxFast {
    #[inline]
    fn compute(a: f32) -> f32 {
        fastapprox::fast::ln(a)
    }
}

#[crate::variant(group = "log")]
pub struct FastapproxFaster;
impl LogVariant for FastapproxFaster {
    #[inline]
    fn compute(a: f32) -> f32 {
        fastapprox::faster::ln(a)
    }
}
