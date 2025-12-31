//! Build script to compile C implementations.

use std::env;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(c_implementation_active)");
    // Check for C compiler compatibility and type
    let build = cc::Build::new();
    let compiler = build.get_compiler();
    let is_gnu_like = compiler.is_like_gnu() || compiler.is_like_clang();
    let is_msvc = compiler.is_like_msvc();
    
    if is_gnu_like || is_msvc {
        let mut compiler_name = "Unknown";
        let mut allow_c_impl = true;
        
        if compiler.is_like_clang() {
            // Check if it's Apple Clang (allowed) or vanilla Clang (disallowed on Linux as per user request)
            // We use a heuristic: if we are on macOS, we assume Apple Clang (ok). 
            // If on Linux/Windows and it's Clang, we mark it as "Clang" and per user rule: "je ne veux pas clang" (implied vanilla).
            let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
            
            if target_os == "macos" {
                compiler_name = "Apple Clang";
            } else {
                compiler_name = "Clang";
                // User requested to disable generic Clang because it's too similar to Rust's LLVM backend
                println!("cargo:warning=Vanilla Clang detected. C implementation disabled as per configuration (requires GCC, MSVC, or Apple Clang).");
                allow_c_impl = false;
            }
        } else if compiler.is_like_gnu() {
            compiler_name = "GCC";
        } else if is_msvc {
            compiler_name = "MSVC";
        }

        if allow_c_impl {
            let rustflags = env::var("RUSTFLAGS").unwrap_or_default();
            let encoded_rustflags = env::var("CARGO_ENCODED_RUSTFLAGS").unwrap_or_default();
            let is_rust_native = rustflags.contains("target-cpu=native") || encoded_rustflags.contains("target-cpu=native");

            let mut build = cc::Build::new();
            
            // Auto-detect all C files in src/ directory
            let c_files = glob::glob("src/**/*.c")
                .expect("Failed to read glob pattern")
                .filter_map(|entry| entry.ok());

            for file in c_files {
                println!("cargo:rerun-if-changed={}", file.display());
                build.file(file);
            }
            
            build
                .opt_level(3)
                .flag_if_supported("-ffast-math"); // Fast math is generally good for SIMD benchmarks

            if is_rust_native {
                build.flag_if_supported("-march=native");
                println!("cargo:warning=Detected Rust target-cpu=native. Enabling -march=native for C compilation.");
            } else {
                 println!("cargo:warning=Rust target-cpu=native NOT detected. Disabling -march=native for C compilation to match Rust baseline.");
            }

            build.compile("dot_product_c");
            
            println!("cargo:rustc-cfg=c_implementation_active");
            println!("cargo:rustc-env=C_COMPILER_NAME={}", compiler_name);
        }
    } else {
        println!("cargo:warning=C compiler is not compatible (needs GCC, Clang, or MSVC). C implementations disabled.");
    }
}
