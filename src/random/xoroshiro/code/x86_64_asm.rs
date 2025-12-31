use std::arch::asm;

pub fn xoroshiro_x86_64_asm(seed_lo_ptr: &mut u64, seed_hi_ptr: &mut u64) -> u64 {
    let mut seed_lo = *seed_lo_ptr;
    let mut seed_hi = *seed_hi_ptr;
    let n;

    unsafe {
        asm!(
            "mov {copy_seed_lo}, {seed_lo}", // create a temporary copy of seed_lo
            "mov {copy_seed_hi}, {seed_hi}", // create a temporary copy of seed_hi
            "xor {seed_hi}, {copy_seed_lo}", // seed_hi = seed_hi ^ seed_lo
            "lea rax, [{copy_seed_lo} + {copy_seed_hi}]", // n = seed_lo + seed_hi
            "rol {seed_lo}, 49", // seed_lo = rol(seed_lo, 49)
            "mov r8, {seed_hi}", // create a copy for ^ (hi << 21)
            "shl r8, 21", // (hi << 21)
            "rol rax, 17", // n = rol(n, 17)
            "xor {seed_lo}, {seed_hi}", // seed_lo ^= seed_hi
            "add rax, {copy_seed_lo}", // n += temp copy of seed_lo (Original Xoroshiro++ involves adding s0? wait check algo)
            // Xoroshiro128++: result = rotl(s0 + s1, 17) + s0;
            // Line 8: lea rax, [s0 + s1] -> n = s0 + s1
            // Line 12: rol rax, 17 -> n = rotl(n, 17)
            // Line 14: add rax, {copy_seed_lo} -> n = n + s0. Correct.
            
            "xor {seed_lo}, r8", // seed_lo ^= (hi << 21) // b ^= c in diagram?
            "rol {seed_hi}, 28", // seed_hi = rol(seed_hi, 28)
            
            seed_lo = inout(reg) seed_lo,
            seed_hi = inout(reg) seed_hi,
            out("rax") n,
            copy_seed_lo = out(reg) _,
            copy_seed_hi = out(reg) _,
            out("r8") _,
            options(nostack, nomem, preserves_flags),
        );
    }

    *seed_lo_ptr = seed_lo;
    *seed_hi_ptr = seed_hi;
    n
}