# Benchmarking Methodology and Design Choices

This document outlines the philosophy, technical implementation, and design choices behind the performance measurement system used in `micro-optimize-algo`.

## Core Philosophy

Micro-benchmarking is notoriously difficult due to system noise, compiler optimizations, and hardware states (e.g., thermal throttling, turbo boost). Our approach aims to mitigate these factors through **statistical rigor** and **randomized execution**.

## Measurement Strategy

### 1. High-Precision Timing

We support two modes of time measurement, controlled via Cargo features:

*   **CPU Cycles (Default)**: Uses hardware counters (e.g., `RDTSC` on x86_64, `CNTVCT_EL0` on aarch64) to measure exact CPU cycles. This is the preferred mode for micro-benchmarking.
*   **Wall-Clock Time (`use_time` feature or `--no-default-features`)**: Uses `std::time::Instant`, which provides monotonically increasing clock ticks. More portable but less precise for small measurements.

#### Why Use CPU Cycles Instead of Wall-Clock Time?

For **micro-benchmarking**, wall-clock time has several limitations:

1.  **Resolution**: `std::time::Instant` typically has microsecond resolution. A function completing in 10-50 nanoseconds cannot be reliably measured—you'd measure mostly timer overhead.
2.  **System Call Overhead**: On some platforms, reading the system clock involves a system call, adding ~100-1000 cycles of overhead per measurement.
3.  **Frequency Scaling**: Wall-clock time depends on actual elapsed time, which is affected by CPU frequency changes (turbo boost, power saving). A function might report 100ns at 3GHz but 150ns at 2GHz, even though it executes the same number of cycles.
4.  **Comparability**: CPU cycles are a hardware-invariant metric. "This function takes 50 cycles" is directly comparable across runs, while "50ns" depends on current clock speed.

**When to use each:**
*   Use **CPU cycles (default)** for functions taking <1µs, or when comparing instruction-level optimizations.
*   Use **wall-clock time** (`--features use_time`) for functions taking >1µs, or when measuring real-world latency matters.

### 2. Execution Protocol

Unlike simple benchmark loops that run `Variant A` 10,000 times, then `Variant B` 10,000 times, we employ a **fully randomized interleaved execution** strategy.

*   **Task Atomicity**: Each benchmark run (e.g., one calculation of a dot product) is treated as a discrete atomic "Task".
*   **Shuffling**: If we have 3 algorithms and want 1,000 iterations each, we generate 3,000 tasks and shuffle them using the Fisher-Yates algorithm.
*   **Impact**: This interleaving ensures that transient system noise (OS interrupts, context switches) affects all algorithms equally distribution-wise, rather than penalizing only the algorithm that happened to be running during a spike.

### 3. Warmup Phase
Before data collection begins, a strict warmup phase executes each algorithm multiple times. This ensures:
*   CPU instruction caches (I-Cache) and data caches (D-Cache) are populated.
*   Branch predictors are trained.
*   The OS pager has mapped necessary memory pages.
*   On JIT-compiled languages (if we were to add them), this would trigger compilation, though Rust is AOT.

## Statistics and Metrics

We report several metrics to give a complete picture of performance stability, not just raw speed.

### Average Time & Speedup
*   **Average**: Arithmetic mean of all iteration times.
*   **Speedup**: Calculated as `Baseline_Time / Variant_Time`. The first variant ("original") is always the baseline (1.0x).

### Coefficient of Variation (CV)
We use the CV to measure the **stability** of the benchmark.
*   **Formula**: $CV = \frac{\sigma}{\mu} \times 100\%$
    *   $\sigma$: Standard Deviation (calculated using unbiased sample variance, $N-1$).
    *   $\mu$: Mean time.
*   **Interpretation**:
    *   **< 1%**: Extremely stable, highly reliable micro-benchmark.
    *   **1% - 5%**: Accepted noise level for system-level benchmarks.
    *   **> 5%**: High variance, suggests interference or multimodal behavior (e.g., garbage collection pauses or context switches).

### Correctness Checks
Performance is meaningless without correctness.
*   **Rel. Error**: We compare the numerical output of optimized variants against the baseline.
*   Any deviation is reported, ensuring that SIMD or bit-twiddling optimizations haven't sacrificed precision.

## Reproducibility

*   **Seeded Randomness**: The benchmark runner accepts a `--seed` option. This seeds the RNG used for input generation (e.g., random vectors) and the execution order shuffler. This allows consistent reproduction of specific "lucky" or "unlucky" run orders during debugging.

## CPU Pinning Strategies

When measuring CPU cycles, **thread migration is a major source of variance**. If the OS scheduler moves your thread to a different core mid-benchmark:

1.  **TSC Desynchronization**: Each core has its own Time Stamp Counter. While modern CPUs synchronize TSCs, there can still be small offsets between cores.
2.  **Cache Invalidation**: Moving to a new core means cold L1/L2 caches. Data and instructions must be re-fetched, adding hundreds of cycles of noise.
3.  **NUMA Effects**: On multi-socket systems, migrating to a core on a different socket means memory accesses go through the interconnect, drastically increasing latency.

To eliminate this variance, we **pin the benchmark thread to a single core** using OS-specific APIs (`sched_setaffinity` on Linux, `SetThreadAffinityMask` on Windows).

**Pinning is enabled by default** since `cpu_cycles` is the default measurement mode.

### Pinning Modes (`--pin`)

*   **`per-call` (Default)**: Pin before each measurement, unpin after. Most accurate but adds overhead.
*   **`global`**: Pin once at session start. Lower overhead but may cause thermal throttling on long runs.

When using wall-clock time (`--features use_time`), the `--pin` option has no effect.

## Data Export

For deeper analysis, the system supports exporting aggregated timing data to **CSV** (`--csv`). The export includes average times per variant and input size, allowing users to perform external analysis such as plotting comparisons or statistical hypothesis testing.
