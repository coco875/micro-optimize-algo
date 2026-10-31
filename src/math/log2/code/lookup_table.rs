use crate::math::log2::Log2Variant;

#[crate::variant(group = "log2")]
pub struct LookupTable;

impl Log2Variant for LookupTable {
    fn prepare() {
        crate::utils::lookup::prepare();
    }

    #[inline]
    fn compute(a: f32) -> f32 {
        crate::utils::lookup::log2(a)
    }
}
