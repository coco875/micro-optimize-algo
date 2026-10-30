# Dot Product Algorithm

Computes the sum of products of corresponding elements in two vectors:

```
dot(a, b) = Σ(a[i] * b[i]) = a[0]*b[0] + a[1]*b[1] + ... + a[n]*b[n]
```

## Implementations

CLI variant filters use these registered IDs:

| ID | Implementation | Availability |
|---|---|---|
| `Original` | Iterator-based reference using `zip` and `map` | All targets |
| `ScalarOpt` | Unrolled loop with four independent accumulators | All targets |
| `Sse2` | 128-bit SIMD, four floats per iteration | x86_64 |
| `Avx2` | 256-bit SIMD, eight floats per iteration | x86_64 with the `avx2` target feature |
| `COriginal` | C reference | Detected C compiler |
| `CScalarOpt` | C unrolled loop | Detected C compiler |
| `CX86_64Sse2` | C SSE2 implementation | Detected C compiler on x86_64 |

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

### SIMD (`x86_64-avx2`)

Uses AVX2 256-bit registers to process 8 `f32` values simultaneously:

```rust
let a_vec = _mm256_loadu_ps(a.as_ptr().add(i));
let b_vec = _mm256_loadu_ps(b.as_ptr().add(i));
sum_vec = _mm256_add_ps(sum_vec, _mm256_mul_ps(a_vec, b_vec));
```

The implementation is compiled only for x86_64 with the `avx2` target feature enabled. It uses `_mm256_fmadd_ps` instead when the `fma` target feature is also enabled. On a supporting machine, `RUSTFLAGS="-C target-cpu=native"` enables the host's features; the Cargo feature named `avx2` alone does not enable this target feature.

## Historical benchmark results

These approximate results predate the runner refactor. The original machine, seed, compiler flags, and exact SIMD variant were not recorded here. They have not been revalidated with the current setup and timing boundaries; rerun the benchmark before using them for performance comparisons.

| Size | Original | x86_64 | Speedup |
|------|----------|--------|---------|
| 256 | ~70ns | ~40ns | 1.75x |
| 1024 | ~370ns | ~175ns | 2.1x |
| 4096 | ~1.5µs | ~700ns | 2.2x |
| 16384 | ~6µs | ~2.7µs | 2.2x |

## Usage

```bash
# Validate both workloads without timing
cargo run --release -- --verify --scenario dot_product --seed 42

# Benchmark both workloads with reproducible inputs and execution order
cargo run --release -- --scenario dot_product --runs 30 --seed 42
```

For custom sizes, save this as `dot-product.toml`:

```toml
sample = 30
seed = 42
filter_scenario = ["dot_product"]

[dot_product]
sizes = [512, 2048, 8192]
```

```bash
cargo run --release -- --config dot-product.toml
```

## Workloads and validation

The `random` scenario uses random vector pairs; `cancellation` alternates large positive and negative terms with small terms. Each implementation receives equivalent logical inputs, prepared before timing. Only the dot-product computation is measured.

Dedicated validation cases check reference parity and an independent f64 product-sum oracle with a rounding budget that accounts for input magnitude and length. Validation sizes are defined by the scenarios independently of the benchmark sizes in TOML.

See [BENCHMARKING.md](../../../BENCHMARKING.md) for timing units, validation, and replay behavior.
