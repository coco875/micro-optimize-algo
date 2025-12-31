/**
 * Dot Product - C Optimized Scalar Implementation
 * 
 * C implementation with 4x loop unrolling.
 */

#include <stddef.h>

float dot_product_c_scalar_opt(const float* a, const float* b, size_t len) {
    float sum0 = 0.0f;
    float sum1 = 0.0f;
    float sum2 = 0.0f;
    float sum3 = 0.0f;
    
    size_t chunks = len / 4;
    size_t remainder = len % 4;
    
    for (size_t i = 0; i < chunks; i++) {
        size_t idx = i * 4;
        sum0 += a[idx] * b[idx];
        sum1 += a[idx + 1] * b[idx + 1];
        sum2 += a[idx + 2] * b[idx + 2];
        sum3 += a[idx + 3] * b[idx + 3];
    }
    
    size_t base = chunks * 4;
    for (size_t i = 0; i < remainder; i++) {
        sum0 += a[base + i] * b[base + i];
    }
    
    return (sum0 + sum1) + (sum2 + sum3);
}
