use crate::math::cos::CosVariant;

#[crate::variant(group = "cos")]
pub struct LookupTable;

impl CosVariant for LookupTable {
    fn prepare() {
        crate::utils::lookup::prepare();
    }

    #[inline]
    fn compute(a: f32) -> f32 {
        crate::utils::lookup::cos(a)
    }
}
