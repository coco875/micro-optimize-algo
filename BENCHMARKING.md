# Benchmarking and validation

## Separate execution paths

`cargo test` and `micro-algo --verify` execute dedicated validation fixtures. Each selected implementation runs once per scenario/case/seed, and several typed checks consume its captured output. Reference-dependent checks share one reference execution; quality checks can operate without a reference.

Benchmark mode collects only timing samples. It does not prepare validation fixtures, capture validation batches, evaluate policies, or claim that results are correct. Run validation separately before interpreting performance.

## Timed work

The generated dispatcher prepares logical inputs and creates fresh implementation-specific state before constructing the timer. It calls the statically specialized scenario's `task`, stops the timer, and releases state afterward. Input generation, allocation/setup, scheduling, statistics, and validation are outside the measurement. An RNG implementation being benchmarked executes inside the timer because generation is its intended workload.

The runner globally shuffles all selected scenario/implementation/parameter/sample tasks. It retains the existing interleaved strategy to distribute transient machine noise across implementations.

Before measurement, every selected scenario/implementation/parameter combination runs 10 warmup tasks with fresh state. Warmup executes the same workload without a timer or validation hooks; its results are excluded from statistics and CSV. Set `--warmup N` or TOML `warmup = N` to change the count; 0 disables it. Warmup uses separate fixture and schedule streams, so changing its count does not change measured inputs or their execution order.

The runner chooses one CPU core before warmup and uses it for both warmup and measurement. `--pin per-call` (the default, also spelled `per-execution`) pins and restores affinity around each task. `--pin global` holds affinity across the entire warmup and measurement phase, then restores it before analysis. TOML accepts `pin = "per-call"` or `pin = "global"`; CLI options override TOML. Pinning is attempted with both counter and wall-clock timing. Affinity changes happen outside the timer, and the guard restores the original affinity on normal return or panic. If pinning is unavailable, including on macOS, the runner reports that and continues unpinned.

Inputs and outputs use `black_box` where needed to keep the intended computation observable to the optimizer. No dynamic dispatch is added to implementation calls by registration; a system can still intentionally benchmark virtual calls as part of its contract.

Gradient dot-product implementations have no internal optimizer barriers. The shared scenario protects inputs and the final result uniformly for scalar and SIMD variants. C code is compiled at optimization level 3 without adding `-ffast-math`; native targeting is enabled when Rust uses `target-cpu=native`.

## Measurement units

- Default `cpu_cycles`: x86_64 TSC measurements; aarch64 virtual-counter ticks (not CPU core cycles).
- `--features use_time` or `--no-default-features`: wall-clock nanoseconds through `Instant`.

Small tasks can be dominated by timer overhead or counter resolution. Interpret results with the reported unit and machine conditions. Random execution order cannot eliminate thermal effects, interrupts, or cache differences caused by setup.

The runner does not automatically batch repeated operations inside a timed sample: repeated access could change cache behavior and the workload being studied. Tiny operations such as a single RNG step therefore still include substantial timer overhead. The aarch64 counter reader has a TODO to add and validate appropriate ISB synchronization on ARM hardware; its instruction sequence is unchanged pending that validation.

## Randomness and replay

Benchmarks use a fresh root seed when omitted and print it. Tests and `--verify` default to root seed 0. Set `--seed N` or TOML `seed = N` to replay.

The central RNG is SplitMix64. Seed derivation applies FNV-1a to the root seed's little-endian bytes and each length-prefixed UTF-8 domain key, followed by the SplitMix64 mixing function:

- Benchmark input: root + `input`, family ID, scenario ID, unique parameter label, sample number.
- Validation input: root + `validation`, family ID, scenario ID, unique validation case ID.
- Scheduling: root + `schedule`.
- Warmup input: root + `warmup`, family ID, scenario ID, unique parameter label, warmup number.
- Warmup scheduling: root + `warmup`, `schedule`.

Fixture seeds exclude implementation IDs and discovery indices. All implementations receive equivalent logical inputs for a case/sample. Filtering or adding implementations and unrelated scenarios leaves an existing fixture unchanged. Random consumption during preparation cannot change the global shuffle. The scheduler uses unbiased Fisher-Yates sampling.

Replay reproduces fixtures and execution order for the same configuration, source, target/features, and available implementations. Changing the selected set changes the schedule; machine noise changes measured times.

## Statistics and export

The default is 30 samples per selected implementation/parameter; `--runs N` overrides it. Reports show arithmetic mean, median, p95, retained sample count, an approximate 95% confidence margin for the mean, and coefficient of variation (CV). Median and p95 use linear interpolation at `(n - 1) * p`. Standard deviation uses sample variance with denominator `n - 1` and retains floating-point precision. Speedup is the ratio of means relative to the first displayed implementation; original implementations sort first when selected.

Mean confidence intervals use `mean +/- t * stddev / sqrt(n)` with conservative Student-t critical values, following the [standard confidence interval for a mean](https://www.itl.nist.gov/div898/handbook/eda/section3/eda352.htm). They assume independent observations and an approximately normal sampling distribution of the mean. Thermal drift, correlated samples, and skewed short runs can violate those assumptions. The intervals are not confidence intervals for speedup and are not a significance test. With one sample, standard deviation, CV, and the interval are unavailable; CV is also unavailable for a zero mean. Console output uses `N/A` and CSV leaves unavailable fields empty.

`--filter` trims each end by `ceil(samples * 0.005)` for groups larger than 10 samples. Descriptive statistics and intervals then describe the retained samples; they do not estimate the unfiltered workload's mean without selection bias. Raw export preserves all observations, including trimmed ones. Validation is a separate path and is never filtered.

`--csv results.csv` writes two files:

- `results.csv`: one row per scenario/implementation/parameter, with columns `group,scenario,variant,context,avg_measurement,seed,unit,samples,total_samples,median_measurement,p95_measurement,min_measurement,max_measurement,stddev_measurement,cv,ci95_low,ci95_high`. `samples` is the retained count, `total_samples` is the count before trimming, and `cv` is a ratio (the console shows percent). All measurement fields and interval bounds use the stated unit. Fractional means and statistics are preserved.
- `results.samples.csv`: one row per measured execution, in execution order, with columns `group,scenario,variant,context,measurement,seed,fixture_seed,unit,execution_index,included`. `execution_index` starts at zero; `included` records whether filtering retained the sample. Warmup executions are excluded. Replacing the summary path's extension with `.samples.csv` determines the companion path.

The `scenario` column contains the stable scenario ID. `seed` is the replay root seed; `fixture_seed` identifies the input shared by matching implementations. Validation reports show scenario, implementation, case, fixture seed, check name, measured value, threshold, and failure reason.
