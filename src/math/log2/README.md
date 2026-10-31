# Log2

`BitApprox` preserves the supplied `log2(f32) -> f32` bit approximation.
`Std` calls `f32::log2` and serves as the reference.

Three additional variants split positive normal inputs into an integer exponent
and a mantissa `m` in `[1, 2)`, then add an approximation of `log2(m)`:

- `Linear`: `m - 1`, interpolating between consecutive powers of two.
- `Quadratic`: `t * (1.33985 - 0.33985*t)`, where `t = m - 1`, interpolating
  `log2(m)` at 1, 1.5, and 2 (coefficients rounded to f32).
- `AtanhSeries`: `2*log2(e)*(z + z^3/3 + z^5/5 + z^7/7 + z^9/9)`, where
  `z = (m - 1)/(m + 1)`.

These three use `f32::log2` for subnormal inputs, negative inputs, zeros,
infinities, and NaNs. Dedicated tests check that fallback, exact results for all
normal powers of two, and sampled approximation error budgets. The supplied
`BitApprox` remains unchanged.

`LookupTable` adds linear interpolation over 4096 mantissa intervals, then adds
the input's exact integer exponent. It uses the shared table implementation
described in [scalar math comparisons](../README.md#lookup-tables), with table
construction excluded from timing even when warmups are disabled. Subnormal,
negative, zero, and nonfinite inputs fall back to `f32::log2`.

External library variants use pinned dependency versions and call the libraries
directly, without adding special-input handling:

| Variant | Library call | Version |
|---|---|---|
| `Libm` | [`libm::log2f`](https://docs.rs/libm/0.2.16/libm/fn.log2f.html) | 0.2.16 |
| `Micromath` | [`micromath::F32Ext::log2`](https://docs.rs/micromath/2.1.0/micromath/trait.F32Ext.html#tymethod.log2) | 2.1.0 |
| `FastapproxFast` | [`fastapprox::fast::log2`](https://docs.rs/fastapprox/0.3.1/fastapprox/fast/fn.log2.html) | 0.3.1 |
| `FastapproxFaster` | [`fastapprox::faster::log2`](https://docs.rs/fastapprox/0.3.1/fastapprox/faster/fn.log2.html) | 0.3.1 |

The micromath wrapper uses a fully qualified trait call so Rust does not select
the inherent `f32::log2` method instead. The approximation libraries have their
own behavior outside the positive normal input domain; they are not covered by
the std fallback policy of our three alternative approximations.

```bash
cargo run --release -- --scenario log2 --runs 30 --seed 42
cargo run --release -- --verify --scenario log2 --seed 42
cargo run --release -- --verify --scenario log2 --seed 42 --csv log2-accuracy.csv
```

The random workload compares identical inputs in two distributions: positive
normal floats with uniformly sampled exponent fields (1 through 254) and mantissa
bits, and floats uniformly sampled in `[1, 2)`. Each timed task processes the
reported number of inputs with identical optimizer barriers for both variants.
Fixture generation and allocation happen outside timing. Configure batch sizes
with `[log2] sizes = [64, 256, 1024, 4096, 16384]` in the TOML configuration.

Validation reports maximum absolute error, root mean squared error, and the
number of unequal results against `Std` over 65,536 inputs per distribution.
These accuracy metrics are informational: PASS only requires matching nonempty
output lengths and finite results on these positive normal inputs. It does not
assert mathematical correctness or a useful approximation tolerance.

With `--verify --csv`, each distribution/implementation gets a CSV row with
numeric columns `metric_max_abs`, `metric_rmse`, `metric_mismatches`, and
`metric_samples`, plus the fixture seed, check result, and description. Values
retain full numeric precision; the console rounds error metrics to eight decimal
places. Timing exports use the benchmark command without `--verify`.

To graph the actual Rust outputs and absolute differences (requires Python with
Matplotlib):

```bash
mkdir -p target/log2-plots
cargo run --release --example log2_samples > target/log2-plots/samples.csv
python3 scripts/plot_log2.py target/log2-plots/samples.csv
```

This creates PNG and SVG graphs: `log2-comparison` covers inputs from 0.25 to 8
and the full positive normal f32 range; `log2-near-one` zooms into `[1, 2)`.
`log2-alternatives` compares all variants over `[1, 2)` with logarithmic error
axes so the more accurate approximations remain visible.
The CSV contains pointwise outputs and signed/absolute differences. This is a
deterministic grid, separate from the random validation sample above. Every
sampled exponent includes its exact power of two and the float immediately
below the next power, so discontinuities are visible.

The supplied function matches normal powers of two, but adding mantissa bits to
the floating-point encoding of the exponent does not compute the logarithm of
the mantissa. Examples and special-value differences are covered by unit tests:

| Input | BitApprox | Std (approximately) |
|---|---|---|
| `1.5` | `5.877472e-39` | `0.5849625` |
| `0.75` | `-1.5` | `-0.4150375` |
| `+0.0` | `-infinity` | `-infinity` |
| `-0.0` | `NaN` | `-infinity` |
| `+infinity` | `128.0` | `+infinity` |
| positive `NaN` | finite | `NaN` |
| smallest positive subnormal | about `-127` | `-149` |

Timings therefore compare different numerical behavior; a speed advantage does
not imply interchangeability with the standard library.

Measured on an Intel Core i7-14700KF (x86_64 Linux), using
`rustc 1.100.0-nightly (c656540d6 2026-08-21)` and the default release profile,
with seed 42, 30 samples, 10 warmups, and per-call pinning to core 8.
For batches of 16,384 inputs, mean TSC cycles per input (including loop and
optimizer-barrier overhead) were:

| Distribution | BitApprox | Std | Std / BitApprox |
|---|---:|---:|---:|
| Positive normal | 1.91 | 6.34 | 3.33x |
| `[1, 2)` | 1.91 | 6.33 | 3.31x |

The validation command above reported these errors for `BitApprox`:

| Distribution | Maximum absolute error | RMSE | Unequal results |
|---|---:|---:|---:|
| Positive normal | 126.59642792 | 38.92537356 | 65,536 / 65,536 |
| `[1, 2)` | 0.99997538 | 0.62501588 | 65,536 / 65,536 |

`Std` has zero error against itself. Measurements are specific to this machine
and workload. To export summary statistics and individual timing samples, add
`--csv /tmp/log2-results.csv` to the benchmark command.

A subsequent comparison of all five variants on the same i7-14700KF used
100 samples, seed 42, 10 warmups, and per-call pinning to core 12, with no
concurrent build or test work. For batches of 16,384 positive normal inputs:

| Variant | Mean TSC cycles/input | Std time / variant time | Maximum sampled absolute error | RMSE |
|---|---:|---:|---:|---:|
| BitApprox | 1.91 | 3.27x | 126.59642792 | 38.92537356 |
| Linear | 2.24 | 2.80x | 0.08607483 | 0.06288300 |
| Quadratic | 2.84 | 2.20x | 0.00866699 | 0.00545319 |
| AtanhSeries | 6.62 | 0.95x | 0.00000763 | 0.00000103 |
| Std | 6.26 | 1.00x | 0 (reference) | 0 (reference) |

Accuracy uses the independent validation fixtures with 65,536 inputs per
distribution. On `[1, 2)`, maximum sampled errors were 0.08607131 (`Linear`),
0.00866269 (`Quadratic`), and 0.00000179 (`AtanhSeries`). Sampled errors are not
proofs of worst-case bounds. The series was slightly slower than std on this
workload, while the quadratic exchanged about 0.00867 absolute error for a
2.20x timing advantage. Subnormal and special-input fallback costs are not
represented by these positive normal timing workloads.

Reproduce and export the comparison:

```bash
cargo run --release -- --scenario log2 --runs 100 --seed 42 --csv target/log2-plots/variants-timing.csv
cargo run --release -- --verify --scenario log2 --seed 42 --csv target/log2-plots/variants-accuracy.csv
```

The external-library comparison used the same CPU and toolchain, 100 samples,
seed 42, 10 warmups, and batches of 16,384 positive normal inputs. On this hybrid
CPU, `taskset -c 12` selected core 12 before launching the benchmark, with the
runner's default per-call pinning. Mean TSC cycles/input include the common
optimizer barriers and loop overhead:

| Variant | Mean TSC cycles/input | Std time / variant time | Maximum sampled absolute difference from std | RMSE |
|---|---:|---:|---:|---:|
| BitApprox | 1.90 | 2.97x | 126.59642792 | 38.92537356 |
| Linear | 2.26 | 2.49x | 0.08607483 | 0.06288300 |
| Quadratic | 2.83 | 2.00x | 0.00866699 | 0.00545319 |
| AtanhSeries | 6.59 | 0.86x | 0.00000763 | 0.00000103 |
| FastapproxFast | 2.80 | 2.02x | 0.00016022 | 0.00009054 |
| FastapproxFaster | 1.33 | 4.23x | 0.05730057 | 0.02572225 |
| Libm | 10.60 | 0.53x | 0.00000763 | 0.00000024 |
| Micromath | 14.85 | 0.38x | 0.16990662 | 0.08813933 |
| Std | 5.64 | 1.00x | 0 (reference) | 0 (reference) |

These are differences from std, not comparisons against an exact mathematical
oracle. Maximum sampled differences on `[1, 2)` were 0.00000006 (`Libm`),
0.00008839 (`Micromath`), 0.00014663 (`FastapproxFast`), and 0.05729848
(`FastapproxFaster`). In this sample `FastapproxFast` had about 54 times smaller
maximum difference than `Quadratic` at a similar mean runtime. `Micromath` was
much closer to std on `[1, 2)` than on the full positive normal range. Its
implementation uses an approximate inverse for values below 1, so behavior in
that interval differs from its direct polynomial evaluation above 1.

Reproduce the CSVs and the `log2-libraries.png` / `.svg` graph:

```bash
mkdir -p target/log2-plots
cargo build --release
taskset -c 12 target/release/micro-algo --scenario log2 --runs 100 --seed 42 --csv target/log2-plots/libraries-timing.csv
target/release/micro-algo --verify --scenario log2 --seed 42 --csv target/log2-plots/libraries-accuracy.csv
cargo run --release --example log2_samples > target/log2-plots/samples.csv
python3 scripts/plot_log2.py target/log2-plots/samples.csv --timing-csv target/log2-plots/libraries-timing.csv --accuracy-csv target/log2-plots/libraries-accuracy.csv
```

`taskset` is Linux-specific; choose an available core on your machine. Omit that
prefix to use the runner's current-core selection. The graph combines error
curves on `[1, 2)` with a speed/accuracy scatter plot using the validation and
timing CSVs. Without those CSV options it shows error over the full positive
normal range instead. Its logarithmic error curves floor displayed differences
at 1e-9, including exact matches; the underlying CSV preserves the observations.
