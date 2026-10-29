# Micro-Optimize-Algo

A collection of Rust, C, SIMD, and assembly implementations compared under independent workloads.

The framework automatically discovers implementation and scenario files, generates static dispatch, and keeps correctness validation separate from timing.

## Systems

| Family | Category | Workloads |
|---|---|---|
| `dot_product` | math | Random vectors and cancellation-heavy inputs |
| `grad_dot` | math | Gradient-vector dot products |
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

# Select implementations and export aggregated timings
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
| `--runs N`, `--iter N`, `-r N` | Benchmark samples per implementation/parameter; default 10 |
| `--config FILE`, `-c FILE` | TOML configuration; otherwise load `config.toml` if it exists |
| `--scenario LIST` | Comma-separated substrings of family, scenario ID, or display name |
| `--variant LIST` | Comma-separated registered implementation ID substrings, e.g. `Original,ScalarOpt` |
| `--tag LIST` | Comma-separated exact tag matches |
| `--csv FILE` | Export aggregated timing measurements, root seed, and measurement unit |
| `--filter`, `-f` | Trim timing extremes when there are more than 10 samples |
| `--help`, `-h` | Show usage |
| Positional names | Additional scenario filters |

For custom sizes, create a TOML configuration:

```toml
sample = 30
seed = 42
filter_scenario = ["dot_product"]

[dot_product]
sizes = [128, 512, 2048]
```

Benchmark CSV columns are `group,scenario,variant,context,avg_measurement,seed,unit`. The `scenario` column contains the stable scenario ID. Measurements use x86_64 TSC cycles, aarch64 virtual-counter ticks, or wall-clock nanoseconds according to the target and selected features. Timing reports do not claim correctness; run validation separately.

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
