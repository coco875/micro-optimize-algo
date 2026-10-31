use crate::math::log2::Log2Variant;

#[crate::variant(group = "log2")]
pub struct Std;

impl Log2Variant for Std {
    #[inline]
    fn compute(a: f32) -> f32 {
        a.log2()
    }
}
