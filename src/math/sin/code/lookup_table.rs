use crate::math::sin::SinVariant;

#[crate::variant(group = "sin")]
pub struct LookupTable;

impl SinVariant for LookupTable {
    fn prepare() {
        crate::utils::lookup::prepare();
    }

    #[inline]
    fn compute(a: f32) -> f32 {
        crate::utils::lookup::sin(a)
    }
}
