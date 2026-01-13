# Contributing

## Algorithm Structure

Each algorithm follows a standard directory structure:

```
src/<category>/<algorithm>/
├── README.md           # Overview, variants description, key findings
├── mod.rs              # AlgorithmRunner implementation
├── code/
│   ├── mod.rs          # get_variants() function, exports
│   ├── original.rs     # Reference implementation
│   ├── <variant>.rs    # Optimized variants
│   └── c_impl.rs       # Optional: C FFI bindings
└── test/
    └── mod.rs          # Verification tests
```

### Key Components

1. **`code/mod.rs`**: Exports `get_variants()` returning `Vec<VariantInfo<FnType>>` with all available variants.
2. **`mod.rs`**: Implements `AlgorithmRunner` trait for integration with the benchmark runner.
3. **`test/mod.rs`**: Contains `verify_all()` function to validate all variants produce correct results.
4. **`README.md`**: Documents the algorithm, variants, and key findings.

## Adding New Algorithms

1.  Create a new directory in `src/<category>/<algorithm>/`.
2.  Implement the algorithm in Rust (`code/original.rs`) and any optimized variants.
3.  Add `get_variants()` in `code/mod.rs` returning all variant functions.
4.  Implement `AlgorithmRunner` in `mod.rs`.
5.  Add verification tests in `test/mod.rs`.
6.  Register the algorithm in `src/registry.rs`.
7.  Add `README.md` with documentation.

## C Implementations

To add C implementations for an algorithm, follow this standard pattern to ensure cross-platform compatibility and graceful degradation when a C compiler is not available.

### 1. File Structure

*   Place C source files (`.c`) in the `code/` directory.
*   Create a `code/c_impl.rs` file to handle FFI.

### 2. `c_impl.rs` Pattern

Your `c_impl.rs` MUST strictly follow this template:

```rust
//! FFI bindings for C implementations.

#[cfg(c_implementation_active)]
mod ffi {
    extern "C" {
        // Declare C functions here
        pub fn my_algo_c(arg: u32) -> u32;
    }
}

// Safe wrapper
#[cfg(c_implementation_active)]
pub fn my_algo_c_wrapper(arg: u32) -> u32 {
    unsafe { ffi::my_algo_c(arg) }
}

// Availability flag
#[cfg(c_implementation_active)]
pub const C_IMPL_AVAILABLE: bool = true;

#[cfg(not(c_implementation_active))]
pub const C_IMPL_AVAILABLE: bool = false;

// Compiler metadata
#[cfg(c_implementation_active)]
pub const COMPILER_NAME: Option<&str> = Some(env!("C_COMPILER_NAME"));

#[cfg(not(c_implementation_active))]
pub const COMPILER_NAME: Option<&str> = None;

// Stubs (REQUIRED for compilation when C is disabled)
#[cfg(not(c_implementation_active))]
pub fn my_algo_c_wrapper(_arg: u32) -> u32 {
    panic!("C implementation not compiled (requires GCC/Clang)")
}
```

### 3. Registration in `mod.rs`

In your `code/mod.rs`, register the C variants conditionally using `C_IMPL_AVAILABLE`.

```rust
pub mod c_impl;

// ... inside get_variants() ...

if c_impl::C_IMPL_AVAILABLE {
    variants.push(VariantInfo {
        name: "c-algo",
        description: "C Implementation",
        function: c_impl::my_algo_c_wrapper,
    });
}
```

This ensures that:
1.  The project compiles even if no C compiler is found (using stubs).
2.  C variants appear in the benchmark list only when they are actually compiled.
3.  We avoid `unsafe` blocks directly in `mod.rs`.
