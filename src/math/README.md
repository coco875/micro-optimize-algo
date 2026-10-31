# Scalar math comparisons

`cos`, `sin`, `tan`, `pow2`, `pow`, and `log` each compare six scalar f32
implementations: `Std`, `Libm` (0.2.16), `Micromath` (2.1.0),
`FastapproxFast`, `FastapproxFaster` (fastapprox 0.3.1), and `LookupTable`.
`Std` is the declared reference and the displayed timing baseline when selected.
The existing `log2` family has its own implementations and workloads.

| Family | Meaning | Std | libm | micromath | fastapprox (both modes) |
|---|---|---|---|---|---|
| `cos` | Cosine, radians | `a.cos()` | `cosf(a)` | `F32Ext::cos(a)` | `cosfull(a)` |
| `sin` | Sine, radians | `a.sin()` | `sinf(a)` | `F32Ext::sin(a)` | `sinfull(a)` |
| `tan` | Tangent, radians | `a.tan()` | `tanf(a)` | `F32Ext::tan(a)` | `tan(a)` |
| `pow2` | `2^a` | `a.exp2()` | `exp2f(a)` | `F32Ext::powf(2.0, a)` | `pow2(a)` |
| `pow` | `a^b` | `a.powf(b)` | `powf(a, b)` | `F32Ext::powf(a, b)` | `pow(a, b)` |
| `log` | Natural logarithm, base e | `a.ln()` | `logf(a)` | `F32Ext::ln(a)` | `ln(a)` |

Micromath does not expose an exp2 method in its `F32Ext` API, so `pow2` uses
its power function with base 2. Fully qualified calls select micromath rather
than Rust's inherent methods. Cosine and sine always use fastapprox's `full`
entry points, including their argument reduction costs, for both workloads.
Tangent uses its bounded entry point within the supported interval. Library
calls retain their own behavior; wrappers do not substitute std on special
inputs or clamp results.

## Lookup tables

`LookupTable` uses linear interpolation over 4096 intervals (4097 entries per
table). Three tables are shared across all seven families, including `log2`:

- A sine table over one period serves sine, phase-shifted cosine, and tangent
  as their ratio. Argument reduction uses f64 and handles negative angles.
- A fractional base-two exponential table handles `pow2`, combined with an
  exponent assembled from float bits.
- A log2 mantissa table handles `log2`, `log` (conversion to base e), and the
  logarithm stage of `pow`. Power combines the log2 and exp2 tables using
  `a^b = 2^(b*log2(a))`.

The shared table data occupy 49,164 bytes (48 KiB + 12 bytes). Values are
constructed once using standard-library f64 functions rounded to f32. A default
`prepare()` hook in each math contract lets the lookup variant force table
initialization in `ScenarioDefinition::new`, outside the timer. This also works
with `--warmup 0`. Lazy table-access checks and interpolation are included in
the measured computations; initial construction is excluded. First-use latency
outside the benchmark is therefore higher than these steady-state timings.

Lookup functions fall back to the corresponding std function for special
inputs, subnormal inputs where applicable, and these cases outside table use:

- Trigonometric inputs with absolute value greater than 1,000,000 radians.
- Tangent when the interpolated cosine has absolute value below 0.0001.
- `pow2` exponents outside `[-126, 126]`.
- Powers with nonpositive or subnormal bases, nonfinite exponents, or a
  computed base-two exponent outside `[-126, 126]`.

Positive normal logarithms span the full normal f32 range through exponent /
mantissa decomposition. Lookup tables approximate the functions; interpolation
errors and cache behavior still affect the accuracy/performance tradeoff.
Tests cover dense angle and mantissa grids, exponential and power relative-error
budgets, table boundaries, signed zeros, special values, and std fallbacks.

API references: [libm](https://docs.rs/libm/0.2.16/libm/),
[micromath](https://docs.rs/micromath/2.1.0/micromath/trait.F32Ext.html),
[fastapprox](https://docs.rs/fastapprox/0.3.1/fastapprox/fast/index.html).

## Workloads

| Families | Case | Inputs |
|---|---|---|
| `cos`, `sin` | `angles-pi` | Uniform angles from `-pi` to `pi` |
| `cos`, `sin` | `angles-100` | Uniform angles from -100 to 100 radians |
| `tan` | `principal` | Uniform angles from -1.4 to 1.4 radians |
| `tan` | `near-poles` | Both signs of `pi/2 - d`, with uniform `d` from 0.001 to 0.05 |
| `pow2` | `exponents-10` | Uniform exponents from -10 to 10 |
| `pow2` | `exponents-100` | Uniform exponents from -100 to 100 |
| `pow` | `powers-small` | Independent base from 0.125 to 8 and exponent from -4 to 4 |
| `pow` | `powers-wide` | Independent base from 0.01 to 100 and exponent from -8 to 8 |
| `log` | `near-one` | Uniform positive inputs from 0.5 to 2 |
| `log` | `positive-normal` | Uniform exponent fields 1 through 254 and uniform mantissa bits |

Uniform real intervals use the existing InputRng grid with the lower endpoint
included and the upper endpoint nominally excluded; f32 rounding can reach an
upper endpoint. Log's positive-normal case samples bit fields, not a uniform
distribution of real values. All cases target finite standard-library outputs.
The tangent workload stays away from poles; `pow` excludes negative bases, and
`log` excludes zeros, negative numbers, and subnormal inputs. This is not an
exhaustive test of special values or a claim about unsupported library domains.

Each timing task processes a batch of inputs with identical input/output
optimizer barriers for every implementation. Preparation and allocation happen
outside the timer. Default batch sizes are 64, 256, 1024, 4096, and 16384; each
family accepts its own `[cos] sizes = [...]`, `[pow] sizes = [...]`, etc. in TOML.

## Run and export

```bash
# All six new families; the tag does not include log2.
cargo run --release -- --tag scalar-math --runs 100 --seed 42 --csv math-timing.csv
cargo run --release -- --verify --tag scalar-math --seed 42 --csv math-accuracy.csv

# One family and selected implementations.
cargo run --release -- --scenario sin --variant Std,Libm,Micromath --seed 42

# Isolate lookup versus std (includes the log2 family).
cargo run --release -- --scenario cos,sin,tan,pow2,pow,log --variant Std,LookupTable --seed 42
cargo run --release -- --verify --tag scalar-math --variant Std,LookupTable --csv lookup-accuracy.csv
```

Validation observes 65,536 inputs per case and implementation. It reports
maximum absolute difference, RMSE, maximum relative difference, unequal result
count, sample count, zero-reference mismatch count, and nonfinite count. Relative
difference is `abs(actual - std) / abs(std)` for nonzero references only;
`relative_samples` counts those observations. Near zeros, relative differences
can be large even when absolute differences are small. Relative values are
ratios, not percentages. Accuracy CSV columns use the `metric_` prefix.

PASS requires matching nonempty output lengths and finite observations on these
finite-output workloads. Accuracy metrics are informational and do not impose
a tolerance on approximation libraries. Comparisons are against std, not an
exact mathematical oracle or a proof of worst-case error. Dedicated known-answer
tests cover std and libm, and shared tests cover fixture domains and metric
handling, including zero reference values and nonfinite outputs.

## Measured example

On the Intel Core i7-14700KF, with rustc 1.100.0-nightly
(`c656540d6 2026-08-21`), the default release profile, seed 42, 100 samples,
10 warmups, and process affinity fixed to core 12, the `FastapproxFast` variant
produced the following results. Timings use batches of 16,384 inputs and include
loop/barrier overhead; accuracy uses independent 65,536-input validation cases.

| Family | Case | Std time / FastapproxFast time | Max absolute difference | Max relative difference (ratio) |
|---|---|---:|---:|---:|
| cos | angles-pi | 1.69x | 0.00003909 | 0.03138 |
| sin | angles-pi | 1.87x | 0.00003909 | 0.004113 |
| tan | principal | 2.42x | 0.00045347 | 0.001200 |
| pow2 | exponents-10 | 0.81x | 0.04705811 | 0.00007042 |
| pow | powers-small | 1.08x | 1.17700195 | 0.00045559 |
| log | near-one | 2.12x | 0.00010169 | 1.13971 |

Ratios below 1 in the timing column indicate a slower variant than std. Relative
differences near zeros can be large: log's absolute difference remains about
0.000102 in this case despite the maximum relative difference exceeding 1.
These are descriptive measurements on this machine, not claims of statistical
significance or worst-case bounds. The exports in `target/scalar-math/timing.csv`
and `target/scalar-math/accuracy.csv` include all five implementations and both
domains per family; `timing.samples.csv` includes all 30,000 measured tasks.

## Lookup measurements

A subsequent run on the same CPU/toolchain used core 12, seed 42, 100 samples,
10 warmups, 16,384-input timing batches, and independent 65,536-input accuracy
cases, selecting only `Std` and `LookupTable`. Table construction was outside
timing. The following rows use the smaller input domains:

| Family | Case | Std time / LookupTable time | Max absolute difference | Max relative difference (ratio) |
|---|---|---:|---:|---:|
| cos | angles-pi | 1.10x | 0.000000358 | 0.00000199 |
| sin | angles-pi | 1.13x | 0.000000358 | 0.000000534 |
| tan | principal | 0.21x | 0.000000954 | 0.000000433 |
| pow2 | exponents-10 | 0.38x | 0.00006104 | 0.000000119 |
| pow | powers-small | 0.50x | 0.00048828 | 0.000000238 |
| log | near-one | 1.07x | 0.0000000596 | 0.00549 |
| log2 | one-to-two | 1.46x | 0.0000000596 | Not collected by the log2 policy |

These ratios describe this implementation and machine; lookup tables do not
guarantee a speed advantage. Argument reduction, indexing, interpolation, lazy
access checks, and cache traffic are part of their measured cost. Near zero,
log's relative difference can remain noticeable despite a small absolute gap.

On the wider input domains, sine and cosine retained a maximum sampled absolute
difference of about 0.000000358. Near the tangent poles it was about 0.000122.
The wide power and pow2 cases have much larger outputs, so their relative
differences are more informative than their absolute differences. Both log and
log2 had a maximum absolute difference of about 0.00000763 over positive normals.

The full comparisons are exported to `target/scalar-math/lookup-timing.csv`,
`lookup-timing.samples.csv`, and `lookup-accuracy.csv`. These include both domains
of all seven functions. Reproduce them with:

```bash
mkdir -p target/scalar-math
cargo build --release
taskset -c 12 target/release/micro-algo --scenario cos,sin,tan,pow2,pow,log --variant Std,LookupTable --runs 100 --seed 42 --csv target/scalar-math/lookup-timing.csv
target/release/micro-algo --verify --scenario cos,sin,tan,pow2,pow,log --variant Std,LookupTable --seed 42 --csv target/scalar-math/lookup-accuracy.csv
```

`taskset` is Linux-specific; select an available CPU core on your machine or
omit the prefix to use the runner's normal current-core selection.
