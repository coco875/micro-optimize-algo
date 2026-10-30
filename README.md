# Micro-Optimize-Algo

A collection of Rust, C, SIMD, and assembly implementations compared under independent workloads.

The framework automatically discovers implementation and scenario files, generates static dispatch, and keeps correctness validation separate from timing.

## Systems

| Family | Category | Workloads |
|---|---|---|
| `dot_product` | math | Random vectors and cancellation-heavy inputs |
| `grad_dot` | math | Gradient-vector dot products |
| `log2` | math | f32 approximations, libm, micromath, and fastapprox versus `f32::log2`; timing and accuracy reports |
| `xoroshiro` | random | Generator steps; dedicated sequence/state and entropy validation |
| `call_vs_branch` | control_flow | Function calls, branches, and inline computation |
| `elseif_vs_jumptable` | control_flow | Else-if chains, jump tables, and branchless dispatch |
| `vtable_iteration` | control_flow | Several object-pool layouts over identical logical inputs |

Architecture-specific implementations are enabled only on their supported targets/features. C availability follows the detected compiler policy.

## Run

Nightly Rust is selected by `rust-toolchain.toml`.

```bash
# Assert all dedicated validation cases
cargo test

# Validate selected systems without timing repetitions
cargo run --release -- --verify --scenario dot_product

# Benchmark, with fresh inputs and globally shuffled execution
cargo run --release

# Reproduce input fixtures and execution order
cargo run --release -- --scenario dot_product --runs 30 --seed 42

# Select implementations and export statistics plus individual samples
cargo run --release -- --scenario xoroshiro --variant Original --seed 42 --csv results.csv

# Wall-clock timing instead of architecture counters
cargo run --release --features use_time

# Enable native Rust and C optimization
RUSTFLAGS="-C target-cpu=native" cargo run --release
```

| Option | Behavior |
|---|---|
| `--verify` | Run dedicated checks; exit nonzero on failures or empty selection |
| `--seed N` | Root seed; validation defaults to 0, benchmarks choose and print a fresh seed |
| `--runs N`, `--iter N`, `-r N` | Benchmark samples per implementation/parameter; default 30 |
| `--warmup N` | Warmup runs per selected implementation/parameter before measurement; default 10, 0 disables |
| `--pin STRATEGY` | Pin to one chosen CPU core: `per-call` (default; also `per-execution`) or `global` |
| `--config FILE`, `-c FILE` | TOML configuration; otherwise load `config.toml` if it exists |
| `--scenario LIST` | Comma-separated substrings of family, scenario ID, or display name |
| `--variant LIST` | Comma-separated registered implementation ID substrings, e.g. `Original,ScalarOpt` |
| `--tag LIST` | Comma-separated exact tag matches |
| `--csv FILE` | Export timing statistics plus `.samples.csv`; with `--verify`, export validation checks and accuracy metrics |
| `--filter`, `-f` | Trim timing extremes when there are more than 10 samples |
| `--help`, `-h` | Show usage |
| Positional names | Additional scenario filters |

For custom sizes, create a TOML configuration:

```toml
sample = 30
seed = 42
warmup = 10
pin = "global"
filter_scenario = ["dot_product"]

[dot_product]
sizes = [128, 512, 2048]
```

Reports show mean, median, p95, sample count, variability, and approximate 95% confidence intervals for the mean. `--csv results.csv` writes summary statistics to `results.csv` and every measured sample to `results.samples.csv`, including execution order, fixture seeds, and filtering decisions. The summary preserves its initial columns `group,scenario,variant,context,avg_measurement,seed,unit` and appends statistical fields; means retain fractional precision. See [BENCHMARKING.md](BENCHMARKING.md) for column definitions and statistical assumptions.

The `scenario` column contains the stable scenario ID. Measurements use x86_64 TSC cycles, aarch64 virtual-counter ticks, or wall-clock nanoseconds according to the target and selected features. Timing reports do not claim correctness; run validation separately.

To export differences from the standard library for `log2`:

```bash
cargo run --release -- --verify --scenario log2 --seed 42 --csv log2-accuracy.csv
```

The validation CSV includes separate numeric columns `metric_max_abs`, `metric_rmse`, `metric_mismatches`, and `metric_samples`, alongside the implementation, case, fixture seed, and check result.

## Extend

```text
src/<category>/<family>/
  mod.rs                  # Annotated system contract and generated module include
  code/
    original.rs           # Reference implementation
    another.rs            # One-file implementation addition
    mod.rs                # Optional shared helpers/reexports
  scenarios/
    basic.rs              # One-file workload addition
    another.rs            # Another workload over every compatible implementation
  test/mod.rs             # Optional known-answer/implementation-specific tests
```

A new family starts with three files: contract, reference, and scenario. Typed checks support exact parity, numeric tolerances, quality measurements, and custom policies. Multiple checks reuse one observation per implementation/case; dedicated validation fixtures are independent of benchmark repetitions.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the extension interfaces and [BENCHMARKING.md](BENCHMARKING.md) for timing and randomness behavior.

## Assembly extraction

```bash
./scripts/extract_asm.sh dot_product
```

Optimization contributions should include benchmark results and machine specifications. CI runs validation and compilation checks, rather than using noisy timing results as a pass/fail condition.

## License

Copyright (c) 2025 coco875.

This project, including its procedural macros, is licensed under the GNU General Public License, version 3 or (at your option) any later version (`GPL-3.0-or-later`). See [LICENSE](LICENSE) for the full text.
