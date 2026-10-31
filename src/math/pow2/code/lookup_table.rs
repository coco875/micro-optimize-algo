use crate::math::pow2::Pow2Variant;

#[crate::variant(group = "pow2")]
pub struct LookupTable;

impl Pow2Variant for LookupTable {
    fn prepare() {
        crate::utils::lookup::prepare();
    }

    #[inline]
    fn compute(a: f32) -> f32 {
        crate::utils::lookup::pow2(a)
    }
}
