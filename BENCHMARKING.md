# Benchmarking Methodology and Design Choices

This document outlines the philosophy, technical implementation, and design choices behind the performance measurement system used in `micro-optimize-algo`.

## Core Philosophy

Micro-benchmarking is notoriously difficult due to system noise, compiler optimizations, and hardware states (e.g., thermal throttling, turbo boost). Our approach aims to mitigate these factors through **statistical rigor** and **randomized execution**.

## Measurement Strategy

### 1. High-Precision Timing
We support two modes of time measurement, controlled via Cargo features:
*   **Wall-Clock Time (Default)**: Uses `std::time::Instant`, which provides monotonically increasing clock ticks. This is portable and sufficient for most macro-optimizations.
*   **CPU Cycles (`cpu_cycles` feature)**: Uses hardware counters (e.g., `RDTSC` on x86_64) to measure exact CPU cycles. This is critical for micro-optimizations where the overhead of a system call or clock resolution would mask the improvements.

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

*   **Seeded Randomness**: The benchmark runner accepts a generic seed. This seeds the RNG used for input generation (e.g., random vectors) and the execution order shuffler. This allows consistent reproduction of specific "lucky" or "unlucky" run orders during debugging.

## Data Export

For deeper analysis, the system supports exporting raw timing data to **CSV** (`--csv`). This allows users to plot histograms or perform hypothesis testing (e.g., Student's t-test) externally to verify if speedups are statistically significant.
