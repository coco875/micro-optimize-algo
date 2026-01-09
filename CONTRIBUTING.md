# Contributing

## Adding New Algorithms

1.  Create a new directory in `src/<category>/<algorithm>/`.
2.  Implement the algorithm in Rust (`code/original.rs`) and any optimized variants.
3.  Add benchmarks in `bench/mod.rs`.
4.  Register the algorithm in `src/registry.rs`.

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

// ... inside available_variants() ...

if c_impl::C_IMPL_AVAILABLE {
    variants.push(VariantInfo {
        name: "c-algo",
        description: "C Implementation",
        function: c_impl::my_algo_c_wrapper,
        compiler: c_impl::COMPILER_NAME,
    });
}
```

This ensures that:
1.  The project compiles even if no C compiler is found (using stubs).
2.  C variants appear in the benchmark list only when they are actually compiled.
3.  We avoid `unsafe` blocks directly in `mod.rs`.
