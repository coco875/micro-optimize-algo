//! CPU Cycle Counter for precise micro-benchmarking.
//!
//! This module provides architecture-specific cycle counter implementations
//! for x86_64 and aarch64.

/// Read the current CPU cycle counter / timer.
///
/// On x86_64: Uses RDTSC with LFENCE for serialization.
/// On aarch64: Uses CNTVCT_EL0 (virtual timer, accessible from userspace).
#[inline(always)]
pub fn read_cycles() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        read_cycles_x86_64()
    }

    #[cfg(target_arch = "x86")]
    {
        read_cycles_x86()
    }

    #[cfg(target_arch = "aarch64")]
    {
        read_cycles_aarch64()
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64")))]
    {
        compile_error!("cpu_cycles feature requires x86, x86_64, or aarch64 architecture");
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn read_cycles_x86_64() -> u64 {
    use core::arch::x86_64::*;
    unsafe {
        // LFENCE prevents speculative execution from affecting RDTSC
        _mm_lfence();
        let cycles = _rdtsc();
        _mm_lfence();
        cycles
    }
}

#[cfg(target_arch = "x86")]
#[inline(always)]
fn read_cycles_x86() -> u64 {
    use core::arch::x86::*;
    unsafe {
        _mm_lfence();
        let cycles = _rdtsc();
        _mm_lfence();
        cycles
    }
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
fn read_cycles_aarch64() -> u64 {
    // CNTVCT_EL0: Virtual timer counter, accessible from userspace
    // Note: This is a fixed-frequency timer, not actual CPU cycles
    // but provides consistent timing across cores
    let val: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) val);
    }
    val
}

/// Measure cycles for a closure
#[inline(always)]
pub fn measure_cycles<F, R>(mut f: F) -> (u64, R)
where
    F: FnMut() -> R,
{
    let start = read_cycles();
    let result = f();
    let end = read_cycles();
    (end.saturating_sub(start), result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hint::black_box;

    #[test]
    fn test_read_cycles_monotonic() {
        let c1 = read_cycles();
        let c2 = read_cycles();
        let c3 = read_cycles();

        // Should be monotonically increasing (or at least not decreasing much)
        assert!(
            c2 >= c1 || c1 - c2 < 1000,
            "Cycles should be roughly monotonic"
        );
        assert!(
            c3 >= c2 || c2 - c3 < 1000,
            "Cycles should be roughly monotonic"
        );
    }

    #[test]
    fn test_measure_cycles() {
        let (cycles, result) = measure_cycles(|| {
            let mut sum = 0u64;
            for i in 0..10000 {
                sum = black_box(sum.wrapping_add(black_box(i)));
            }
            sum
        });

        // With black_box, cycles should be > 0
        // On some fast CPUs this could still be very small
        assert!(result > 0, "Result should be computed");
        // Don't assert cycles > 0 as CNTVCT_EL0 resolution may be low
        let _ = cycles; // Just ensure it compiles
    }
}
