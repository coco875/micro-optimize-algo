/**
 * Dot Product - C SSE2 Implementation
 * 
 * SSE2 SIMD intrinsics - available on all x86_64 CPUs.
 * Processes 4 floats per iteration.
 */

#include <stddef.h>
#include <xmmintrin.h>  // SSE
#include <emmintrin.h>  // SSE2

float dot_product_c_x86_64_sse2(const float* a, const float* b, size_t len) {
    if (len < 4) {
        float sum = 0.0f;
        for (size_t i = 0; i < len; i++) {
            sum += a[i] * b[i];
        }
        return sum;
    }
    
    size_t chunks = len / 4;
    size_t remainder = len % 4;
    
    __m128 sum_vec = _mm_setzero_ps();
    
    for (size_t i = 0; i < chunks; i++) {
        size_t idx = i * 4;
        __m128 a_vec = _mm_loadu_ps(a + idx);
        __m128 b_vec = _mm_loadu_ps(b + idx);
        __m128 prod = _mm_mul_ps(a_vec, b_vec);
        sum_vec = _mm_add_ps(sum_vec, prod);
    }
    
    // Horizontal sum
    __m128 shuf = _mm_shuffle_ps(sum_vec, sum_vec, _MM_SHUFFLE(1, 1, 3, 3)); // Imitate movehdup with shuffle
    __m128 sums = _mm_add_ps(sum_vec, shuf);
    __m128 shuf2 = _mm_movehl_ps(sums, sums);
    __m128 sums2 = _mm_add_ss(sums, shuf2);
    
    float result = _mm_cvtss_f32(sums2);
    
    // Handle remainder
    size_t base = chunks * 4;
    for (size_t i = 0; i < remainder; i++) {
        result += a[base + i] * b[base + i];
    }
    
    return result;
}
