# Gradient Dot Product (grad_dot)

Computes the dot product of a noise-gradient vector and the corresponding relative position vector. This is a core primitive in multi-dimensional noise generation algorithms (such as Perlin noise and Simplex noise).

Given a hash value (which is used to look up a pre-defined 3D gradient vector $\mathbf{g}$ from a codebook of 16 vectors) and the offset coordinates $x, y, z$, the algorithm calculates:

$$\text{grad\_dot}(\text{hash}, x, y, z) = \mathbf{g}[\text{hash} \bmod 16] \cdot [x, y, z]^T = g_x \cdot x + g_y \cdot y + g_z \cdot z$$

## Implementations

- **original**: Scalar reference implementation that iterates over the vectors and performs the dot product for each coordinate triplet sequentially.
- **simd_portable_x4**: Vectorized implementation using Rust's nightly `portable_simd` feature (`std::simd::f64x4`). It processes four coordinate triplets in parallel, loading the inputs into 256-bit SIMD registers.

## Optimization Strategies

### SIMD Vectorization (`simd_portable_x4`)

Using `std::simd::f64x4`, we can process 4 offset coordinates simultaneously:

1. Look up the gradient vectors for the 4 hash values from the codebook `GRADIENT`.
2. Gather their $x, y, z$ components into three distinct `f64x4` SIMD registers.
3. Load the position coordinates $x, y, z$ as `f64x4` SIMD registers.
4. Perform the fused element-wise multiplication and accumulation:
   $$\text{res} = \mathbf{g}_x \times x + \mathbf{g}_y \times y + \mathbf{g}_z \times z$$

This enables significant performance gains on modern CPU architectures with 256-bit SIMD pipelines (AVX/AVX2).
