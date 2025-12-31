# Micro-Optimize-Algo

A collection of algorithms that have been micro-optimized using various techniques and architectures.

## Overview

This repository provides multiple optimized implementations of common algorithms, primarily written in **Rust** for easy cross-platform reproducibility.

Additionally:
- **C implementations** can be added to compare compiler performance (LLVM vs GCC)
- **Utility files** may be provided to help convert types and structures between Rust and C

### Project Structure

Each algorithm includes its own README explaining the algorithm and potential optimization strategies.

```
src/
├── registry.rs           # Generic algorithm registry
├── math/
│   └── dot_product/
│       ├── README.md     # Algorithm documentation
│       ├── code/         # Implementation variants
│       ├── bench/        # Benchmarks
│       └── test/         # Tests
└── bin/
    └── all.rs            # CLI binary
```

### Available Algorithms

| Algorithm | Category | Variants | Description |
|-----------|----------|----------|-------------|
| `dot_product` | math | original, scalar_opt, x86_64-sse2, x86_64-avx2, c-original, c-scalar_opt, c-x86_64-sse2 | Sum of products of vector elements |
| `xoroshiro128++` | random | original, x86_64-asm, c-original | High-speed pseudo-random number generator |

### Variant Naming Convention

Variant names are built by combining the following components:

| Component | Options | Description |
|-----------|---------|-------------|
| **Language** | `c` | C implementation (default is Rust) |
| **Architecture** | `x86`, `x86_64`, `arm64`, `arm32` | Target CPU architecture |
| **SIMD** | `sse2`, `avx`, `avx2`, `avx512`, `neon` | Vector instruction set |
| **Technique** | `asm`, `parallel` | Additional optimization technique |

**Format:** `[<language>-]<architecture>[-<simd>][-<technique>]`

**Examples:**

| Variant | Meaning |
|---------|---------|
| `original` | Clean, idiomatic Rust reference implementation |
| `scalar_opt` | Optimized scalar implementation (manual loop unrolling) |
| `x86_64-avx2` | x86_64 with AVX2 SIMD intrinsics |
| `c-x86_64-avx2` | C implementation with AVX2 intrinsics |
| `arm64-neon` | ARM64 with NEON SIMD intrinsics |

### Mathematical Functions

For mathematical algorithms (e.g., trigonometry, square roots, logarithms), multiple implementations are provided with:
- **Execution time** measurements
- **Precision analysis** (error margin compared to reference implementation)

## Usage

### Running Benchmarks

Use the CLI to run benchmarks with the custom TUI:

```bash
# Run all algorithms
cargo run --release

# Run a specific algorithm
cargo run --release -- dot_product

# Run with custom sizes and iterations
cargo run --release -- --sizes 1024,8192 --iter 1000

# List all available algorithms
cargo run --release -- --list
```

### Running Tests

Verify the correctness of all algorithms:

```bash
cargo test
```

### Native Optimization

To enable architecture-specific optimizations (AVX2, AVX-512, etc.) for **both** Rust and C implementations:

```bash
RUSTFLAGS="-C target-cpu=native" cargo run --release
```

> **Note:** This automatically detects the flag and enables `-march=native` for the C compiler, ensuring a fair performance comparison.

## Adding a New Algorithm

1. **Create Directory Structure**:
   Follow the standard hierarchy for organization:
   ```
   src/<category>/<algorithm>/
   ├── README.md         # Documentation
   ├── mod.rs            # Main module (Runner implementation)
   ├── code/             # Implementations (variants)
   │   ├── mod.rs        # Exports variants
   │   ├── original.rs   # Reference implementation
   │   └── ...           # Other variants (e.g., x86_64.rs)
   ├── bench/            # Benchmark logic
   │   └── mod.rs
   └── test/             # Unit tests
       └── mod.rs
   ```

2. **Implement the Runner**:
   In `src/<category>/<algorithm>/mod.rs`, create a struct implementing `AlgorithmRunner`:
   ```rust
   pub struct MyAlgoRunner;

   impl AlgorithmRunner for MyAlgoRunner {
       fn name(&self) -> &'static str { "my_algo" }
       fn category(&self) -> &'static str { "math" }
       fn description(&self) -> &'static str { "Computes something fast" }
       fn available_variants(&self) -> Vec<String> { ... }
       fn run_benchmarks(&self, size: usize, iter: usize) -> Vec<BenchmarkResult> { ... }
       fn verify(&self) -> Result<(), String> { ... }
   }
   ```

3. **Define Variants**:
   Create a `Variant` struct or enum in `code/mod.rs` to hold function pointers and metadata (name, description, compiler).

4. **Add C Implementations (Optional)**:
   Simply place `.c` files in the `code/` directory (e.g., `src/<category>/<algorithm>/code/c_impl.c`).
   - The build system automatically detects and compiles all `src/**/*.c` files.
   - Use `build.rs` environment variables (like `C_COMPILER_NAME`) if needed.

5. **Register the Algorithm**:
   Add the new runner to `src/registry.rs`:
   ```rust
   registry.register(crate::<category>::<algorithm>::MyAlgoRunner);
   ```

## Contributing

Contributions are welcome! Please submit a pull request.

**Requirements for optimization PRs:**
- Include benchmark results
- Specify CPU model and RAM specifications

> **Note:** The CI pipeline only runs tests, not benchmarks.