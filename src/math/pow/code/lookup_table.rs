use crate::math::pow::PowVariant;

#[crate::variant(group = "pow")]
pub struct LookupTable;

impl PowVariant for LookupTable {
    fn prepare() {
        crate::utils::lookup::prepare();
    }

    #[inline]
    fn compute(a: f32, b: f32) -> f32 {
        crate::utils::lookup::pow(a, b)
    }
}
