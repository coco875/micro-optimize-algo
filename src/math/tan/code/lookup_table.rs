use crate::math::tan::TanVariant;

#[crate::variant(group = "tan")]
pub struct LookupTable;

impl TanVariant for LookupTable {
    fn prepare() {
        crate::utils::lookup::prepare();
    }

    #[inline]
    fn compute(a: f32) -> f32 {
        crate::utils::lookup::tan(a)
    }
}
