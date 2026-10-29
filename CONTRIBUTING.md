# Contributing

The framework separates a **system contract**, its **implementations**, **scenarios** (workloads), and typed **validation checks**. A system can be a function, stateful object, or pipeline.

## Add an implementation: one file

Place a Rust file directly in `src/<category>/<family>/code/`. Implement the existing family trait and annotate its marker struct:

```rust
use crate::math::dot_product::DotProductVariant;

#[crate::variant(group = "dot_product")]
pub struct MyImplementation;

impl DotProductVariant for MyImplementation {
    fn name() -> &'static str { "my-implementation" }
    fn compute(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b).map(|(a, b)| a * b).sum()
    }
}
```

The file is automatically included, paired with every family scenario, and validated by generated tests. No registry or module-list edits are needed. The registered ID defaults to the struct name; use `id = "..."` to override it. CLI variant filters match registered IDs, preserving existing struct-based names.

For an architecture-specific file, put `#![cfg(target_arch = "x86_64")]` at its beginning. Discovery propagates file and item `cfg` attributes to registrations. Add any required target-feature gate, such as `target_feature = "avx2"`, explicitly. Macro annotations must appear on top-level items; external files must sit directly in the documented directories.

## Add a scenario: one file

Place a Rust file in `src/<category>/<family>/scenarios/`. Define a generic struct annotated with `#[crate::scenario(group = "...", id = "...", name = "...", tags = "...")]`. `id` defaults to the struct name and `name` defaults to that name without its `Scenario` suffix. Give the struct exactly one implementation type parameter.

Implement `Scenario` and `ScenarioDefinition<V>`:

| Interface | Responsibility |
|---|---|
| `Param`, `generate_params`, `format_param` | Benchmark cases and their unique, stable labels |
| `Fixture`, `prepare(param, rng)` | Implementation-independent logical input generated before timing |
| `new(fixture)` | Fresh implementation-specific state, also before timing |
| `task()` | Only the workload being measured; protect inputs/results with `black_box` |
| `validation_cases(config)` | Dedicated, uniquely named correctness cases, independent of timing repetitions |
| `Output`, `validate(fixture)` | One complete observation per implementation and validation case |
| `checks(fixture)` | Several policies consuming that same typed observation |

`Param`, `Fixture`, and `Output` must agree for all implementations of a scenario. Preparation and policy selection are performed through the family's explicit reference specialization and must not depend on the implementation type. Fresh mutable state belongs to `new`/`validate`, not the shared fixture.

See `src/math/dot_product/scenarios/random.rs` and `cancellation.rs` for independent workloads over one contract, and `src/random/xoroshiro/scenarios/sequence.rs` for collecting a batch outside benchmark timing. A system may contain a pipeline; this version selects one interchangeable system per scenario.

## Add a family: three initial files

Create `src/<category>/<family>/mod.rs`, `code/original.rs`, and `scenarios/basic.rs`. The contract module declares a trait and includes the discovered child modules:

```rust
#[crate::algorithm(group = "my_system", reference = "code::original::Original")]
pub trait MySystem: Send + 'static {
    fn compute(input: &[u8]) -> u64;
}

crate::algorithm_modules!("my_category/my_system");
```

The category and family are registered automatically. The reference path is relative to this family and must name an available annotated implementation. An unknown or disabled reference produces a compile error.

An optional `code/mod.rs` can hold shared helpers/reexports. If present, it must include `crate::implementation_modules!("my_category/my_system");` for automatic implementation imports. Existing families use this for their public algorithm function reexports. Do not manually declare discovered implementation or scenario modules.

Build products live in `OUT_DIR`; `src/registry.rs` includes the generated registry. Never edit generated files. Discovery rejects malformed metadata, duplicate IDs, unknown families, missing references, and parse errors. Directory tracking also catches file additions and deletions.

## Add a validation policy

Implement `utils::validation::Check<T>` in an ordinary Rust file and select it from the scenario's `checks` method. New policies require no central enum or registry changes. A helper file placed under `scenarios/` is imported automatically even when it contains no scenario annotation.

```rust
use crate::utils::validation::{Check, CheckResult};

pub struct NonEmpty;
impl Check<Vec<u8>> for NonEmpty {
    fn name(&self) -> &str { "non-empty" }
    fn needs_reference(&self) -> bool { false }
    fn evaluate(&self, actual: &Vec<u8>, _: Option<&Vec<u8>>) -> CheckResult {
        CheckResult::new(!actual.is_empty(), actual.len().to_string(), "> 0", "output length")
    }
}
```

Built-in policies include `Exact`, `NumericTolerance`, and `ByteEntropy`. `Required` adapts a policy to an optional output and rejects missing observations. `Project` applies a policy to one field of a structured output without executing it again. Numeric tolerance uses `abs(actual - expected) <= absolute + relative * abs(expected)`; NaN fails, and only matching infinities pass. An optional numeric oracle makes that check independent of the reference. Empty observations must be rejected explicitly by policies whose measurement requires data; typed outputs avoid the previous optional metric path.

Checks report measured values, thresholds, and reasons. The reference is executed once per validation case and reused, even when filtered out of the selected implementations. Reference-independent checks do not execute an unselected reference. No checks or no validation cases cannot produce a successful validation run.

Dot-product scenarios use an f64 product-sum oracle and an absolute rounding budget of `4 * (n + 1) * f32::EPSILON * sum(abs(products))`. This handles cancellation and alternate reduction orders without relying only on reference parity. Tighten or replace this policy when introducing a different numeric contract.

Xoroshiro validation captures 65,536 outputs and the final state, then checks exact parity and byte-histogram Shannon entropy >= 7.99 bits/byte. Entropy is a deterministic quality smoke check, not a comprehensive randomness certification.

## Validation and benchmarking

```bash
cargo test
cargo run --release -- --verify --scenario dot_product
cargo run --release -- --verify --scenario xoroshiro --variant Original --seed 42
cargo run --release -- --scenario dot_product --runs 30 --seed 42
```

Validation is separate from timing. `--verify` reports all selected checks and exits nonzero on failure or an empty selection. Timing repetitions never invoke validation hooks. Tests and `--verify` use seed 0 by default; benchmarks choose and print a fresh seed unless one is supplied.

Use the provided `InputRng` in fixture preparation. Input, validation, and scheduling streams are separated by stable family/scenario/case identifiers. Implementation filters, new implementations, and unrelated scenarios do not change an existing fixture. Replay reproduces inputs and schedule for the same configuration and enabled implementation set; timings remain subject to machine noise.

## C implementations

C files under `src/` are discovered and compiled using the existing compiler policy. Add Rust wrappers/annotated implementation structs in `code/`, gated with `#[cfg(c_implementation_active)]`. A C implementation generally needs both a `.c` source and a Rust wrapper. Keep unsafe FFI at the wrapper boundary and preserve disabled-C stubs for existing public functions.

## Checks

Use nightly Rust (the toolchain file already selects it). Run `cargo test`, tests with `--features use_time` and `--no-default-features`, and `cargo clippy --all-targets --all-features -- -D warnings`.

Automatically discovered files are not traversed by Cargo formatting. Format/check every source file explicitly:

```bash
rg --files -g '*.rs' -g '!target/**' | xargs rustfmt --edition 2021 --config skip_children=true
rg --files -g '*.rs' -g '!target/**' | xargs rustfmt --edition 2021 --config skip_children=true --check
```
