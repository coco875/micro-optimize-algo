#include <stdint.h>

static inline uint64_t rotl(const uint64_t x, int k) {
	return (x << k) | (x >> (64 - k));
}

uint64_t xoroshiro128plusplus_c(uint64_t *s0_ptr, uint64_t *s1_ptr) {
	uint64_t s0 = *s0_ptr;
	uint64_t s1 = *s1_ptr;
	
    // Xoroshiro128++ algorithm
	const uint64_t result = rotl(s0 + s1, 17) + s0;

	s1 ^= s0;
	*s0_ptr = rotl(s0, 49) ^ s1 ^ (s1 << 21); // a, b
	*s1_ptr = rotl(s1, 28); // c

	return result;
}
