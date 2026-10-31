use crate::math::log::LogVariant;

#[crate::variant(group = "log")]
pub struct LookupTable;

impl LogVariant for LookupTable {
    fn prepare() {
        crate::utils::lookup::prepare();
    }

    #[inline]
    fn compute(a: f32) -> f32 {
        crate::utils::lookup::log(a)
    }
}
