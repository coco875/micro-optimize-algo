# Gradient Dot Product (grad_dot)

Computes the dot product of a noise-gradient vector and the corresponding relative position vector. This is a core primitive in multi-dimensional noise generation algorithms (such as Perlin noise and Simplex noise).

Given a hash value (which is used to look up a pre-defined 3D gradient vector $\mathbf{g}$ from a codebook of 16 vectors) and the offset coordinates $x, y, z$, the algorithm calculates:

$$\text{grad\_dot}(\text{hash}, x, y, z) = \mathbf{g}[\text{hash} \bmod 16] \cdot [x, y, z]^T = g_x \cdot x + g_y \cdot y + g_z \cdot z$$

## Implementations

- **original**: Scalar reference implementation that iterates over the vectors and performs the dot product for each coordinate triplet sequentially.
- **simd_portable_x4**: Vectorized implementation using Rust's nightly `portable_simd` feature (`std::simd::f64x4`) with a loop-based gather of gradient components.
- **simd_transpose_x4**: Advanced vectorized implementation that avoids loop-based gathers by loading entire 4D gradient vectors (`GRADIENT_4`) into SIMD registers and transposing the resulting matrix in-register using `deinterleave`.

## Optimization Strategies

### SIMD Vectorization (`simd_portable_x4` and `simd_transpose_x4`)

Using `std::simd::f64x4`, we can process 4 offset coordinates simultaneously:

1. **Loop Gather (`simd_portable_x4`)**: Looks up the 3D gradient vectors from `GRADIENT` sequentially in a loop and loads them into `f64x4` arrays.
2. **Transpose Gather (`simd_transpose_x4`)**:
   - Loads the 4D gradient vectors `h0, h1, h2, h3` from `GRADIENT_4` directly as `f64x4` registers.
   - Performs a 4x4 matrix transposition entirely in-register using two levels of `deinterleave` calls to gather the components into `gx, gy, gz`.
   - Performs a fused element-wise multiplication and accumulation with the position vectors:
     $$\text{res} = \mathbf{g}_x \times x + \mathbf{g}_y \times y + \mathbf{g}_z \times z$$

The `simd_transpose_x4` variant avoids all memory layout shuffling and temporary buffer stores, yielding a performance improvement of up to ~1.45x compared to the scalar baseline.
