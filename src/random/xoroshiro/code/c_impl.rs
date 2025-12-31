use super::VariantInfo;

extern "C" {
    fn xoroshiro128plusplus_c(s0: *mut u64, s1: *mut u64) -> u64;
}

pub fn xoroshiro_c_wrapper(s0: &mut u64, s1: &mut u64) -> u64 {
    unsafe { xoroshiro128plusplus_c(s0, s1) }
}

pub const VARIANT: VariantInfo = VariantInfo {
    name: "c-original",
    function: xoroshiro_c_wrapper,
    description: "C implementation of Xoroshiro128++",
    compiler: Some(env!("C_COMPILER_NAME")),
};
