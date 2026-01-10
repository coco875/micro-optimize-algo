pub fn xoroshiro_original(seed_lo: &mut u64, seed_hi: &mut u64) -> u64 {
    let s0 = *seed_lo;
    let s1 = *seed_hi;

    // Xoroshiro128++ algorithm
    let result = s0.wrapping_add(s1).rotate_left(17).wrapping_add(s0);

    let s1 = s1 ^ s0;
    *seed_lo = s0.rotate_left(49) ^ s1 ^ (s1 << 21); // a, b
    *seed_hi = s1.rotate_left(28); // c

    result
}
