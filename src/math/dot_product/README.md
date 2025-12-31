# Dot Product Algorithm

Computes the sum of products of corresponding elements in two vectors:

```
dot(a, b) = Σ(a[i] * b[i]) = a[0]*b[0] + a[1]*b[1] + ... + a[n]*b[n]
```

## Implementations

- **original**: Simple iterator-based implementation using `zip` and `map`. Rust's compiler is already very good at optimizing this.
- **scalar_opt**: Manually unrolled loop (4 elements per iteration) with 4 independent accumulators to break dependency chains. This allows the CPU to execute multiple multiplications in parallel (instruction-level parallelism).
- **x86_64-sse2**: Uses SSE2 (128-bit) intrinsics to process 4 floats at once. Available on all x86_64 CPUs.

## Optimization Strategies

### Scalar Optimization (`scalar_opt`)

Processes 4 elements per iteration with separate accumulators to reduce loop overhead and break data dependencies:

```rust
sum0 += a[i] * b[i];
sum1 += a[i+1] * b[i+1];
sum2 += a[i+2] * b[i+2];
sum3 += a[i+3] * b[i+3];
```

**Benefits:**
- Fewer branch predictions
- Better instruction pipelining
- ~2x speedup on large vectors

### SIMD (`x86_64-avx2`)

Uses AVX2 256-bit registers to process 8 `f32` values simultaneously:

```rust
let a_vec = _mm256_loadu_ps(a.as_ptr().add(i));
let b_vec = _mm256_loadu_ps(b.as_ptr().add(i));
sum_vec = _mm256_fmadd_ps(a_vec, b_vec, sum_vec); // FMA
```

**Requirements:** CPU with AVX2 support (Intel Haswell+, AMD Excavator+)

## Benchmark Results

Tested on typical x86_64 hardware:

| Size | Original | x86_64 | Speedup |
|------|----------|--------|---------|
| 256 | ~70ns | ~40ns | 1.75x |
| 1024 | ~370ns | ~175ns | 2.1x |
| 4096 | ~1.5µs | ~700ns | 2.2x |
| 16384 | ~6µs | ~2.7µs | 2.2x |

## Usage

```bash
# Run dot product benchmarks
cargo run --bin micro-algo --release -- dot_product

# Custom sizes
cargo run --bin micro-algo --release -- dot_product --sizes 512,2048,8192
```
