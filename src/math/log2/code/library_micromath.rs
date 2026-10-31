use crate::math::log2::Log2Variant;

#[crate::variant(group = "log2")]
pub struct Micromath;

impl Log2Variant for Micromath {
    #[inline]
    fn compute(a: f32) -> f32 {
        // UFCS selects the library approximation rather than f32's inherent method.
        micromath::F32Ext::log2(a)
    }
}
