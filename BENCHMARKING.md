# Benchmarking and validation

## Separate execution paths

`cargo test` and `micro-algo --verify` execute dedicated validation fixtures. Each selected implementation runs once per scenario/case/seed, and several typed checks consume its captured output. Reference-dependent checks share one reference execution; quality checks can operate without a reference.

Benchmark mode collects only timing samples. It does not prepare validation fixtures, capture validation batches, evaluate policies, or claim that results are correct. Run validation separately before interpreting performance.

## Timed work

The generated dispatcher prepares logical inputs and creates fresh implementation-specific state before constructing the timer. It calls the statically specialized scenario's `task`, stops the timer, and releases state afterward. Input generation, allocation/setup, scheduling, statistics, and validation are outside the measurement. An RNG implementation being benchmarked executes inside the timer because generation is its intended workload.

The runner globally shuffles all selected scenario/implementation/parameter/sample tasks. It retains the existing interleaved strategy to distribute transient machine noise across implementations. There is no automatic warmup phase or CPU pinning in this runner; those utilities remain available for separate use.

Inputs and outputs use `black_box` where needed to keep the intended computation observable to the optimizer. No dynamic dispatch is added to implementation calls by registration; a system can still intentionally benchmark virtual calls as part of its contract.

## Measurement units

- Default `cpu_cycles`: x86_64 TSC measurements; aarch64 virtual-counter ticks (not CPU core cycles).
- `--features use_time` or `--no-default-features`: wall-clock nanoseconds through `Instant`.

Small tasks can be dominated by timer overhead or counter resolution. Interpret results with the reported unit and machine conditions. Random execution order cannot eliminate thermal effects, interrupts, or cache differences caused by setup.

## Randomness and replay

Benchmarks use a fresh root seed when omitted and print it. Tests and `--verify` default to root seed 0. Set `--seed N` or TOML `seed = N` to replay.

The central RNG is SplitMix64. Seed derivation applies FNV-1a to the root seed's little-endian bytes and each length-prefixed UTF-8 domain key, followed by the SplitMix64 mixing function:

- Benchmark input: root + `input`, family ID, scenario ID, unique parameter label, sample number.
- Validation input: root + `validation`, family ID, scenario ID, unique validation case ID.
- Scheduling: root + `schedule`.

Fixture seeds exclude implementation IDs and discovery indices. All implementations receive equivalent logical inputs for a case/sample. Filtering or adding implementations and unrelated scenarios leaves an existing fixture unchanged. Random consumption during preparation cannot change the global shuffle. The scheduler uses unbiased Fisher-Yates sampling.

Replay reproduces fixtures and execution order for the same configuration, source, target/features, and available implementations. Changing the selected set changes the schedule; machine noise changes measured times.

## Statistics and export

Reports show arithmetic mean, minimum, maximum, speedup relative to the first displayed implementation, and coefficient of variation using sample variance. Original implementations sort first when selected. `--filter` trims each end by `ceil(samples * 0.005)` for groups larger than 10 samples, without discarding validation results because those are collected separately.

CSV exports aggregated timing data using `group,scenario,variant,context,avg_measurement,seed,unit`. The `scenario` column contains the stable scenario ID. The seed is the replay root seed. Validation reports show scenario, implementation, case, fixture seed, check name, measured value, threshold, and failure reason.
