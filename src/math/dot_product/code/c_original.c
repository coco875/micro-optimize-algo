/**
 * Dot Product - C Original Implementation
 * 
 * Basic C implementation - lets the compiler optimize.
 */

#include <stddef.h>

float dot_product_c_original(const float* a, const float* b, size_t len) {
    float sum = 0.0f;
    for (size_t i = 0; i < len; i++) {
        sum += a[i] * b[i];
    }
    return sum;
}
