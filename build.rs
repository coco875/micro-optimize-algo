//! Discover Rust workloads and compile optional C implementations.
use std::{env, fs, path::Path};
mod build_support;

fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=macros/src/metadata.rs");
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out = env::var("OUT_DIR").unwrap();
    if let Err(error) = build_support::generate(&Path::new(&manifest).join("src"), Path::new(&out))
    {
        // Surface discovery failures in rustc rather than silently dropping a file.
        fs::write(Path::new(&out).join("algorithms.rs"), "").unwrap();
        fs::write(
            Path::new(&out).join("registry.rs"),
            format!("compile_error!({error:?});"),
        )
        .unwrap();
    }
    compile_c_code();
}

fn compile_c_code() {
    println!("cargo:rustc-check-cfg=cfg(c_implementation_active)");
    let build = cc::Build::new();
    let compiler = build.get_compiler();
    let is_gnu_like = compiler.is_like_gnu() || compiler.is_like_clang();
    let is_msvc = compiler.is_like_msvc();

    if is_gnu_like || is_msvc {
        let mut compiler_name = "Unknown";
        let mut allow_c_impl = true;

        if compiler.is_like_clang() {
            let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
            if target_os == "macos" {
                compiler_name = "Apple Clang";
            } else {
                compiler_name = "Clang";
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
            let is_rust_native = rustflags.contains("target-cpu=native")
                || encoded_rustflags.contains("target-cpu=native");

            let mut build = cc::Build::new();
            let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

            let c_files = glob::glob("src/**/*.c")
                .expect("Failed to read glob pattern")
                .filter_map(|entry| entry.ok())
                .filter(|path| {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if filename.contains("x86_64") && target_arch != "x86_64" {
                        return false;
                    }
                    if filename.contains("aarch64") && target_arch != "aarch64" {
                        return false;
                    }
                    true
                });

            for file in c_files {
                println!("cargo:rerun-if-changed={}", file.display());
                build.file(file);
            }

            println!("cargo:rerun-if-changed=src");
            build.opt_level(3);

            if is_rust_native {
                build.flag_if_supported("-march=native");
            }

            build.compile("micro_algo_c");
            println!("cargo:rustc-cfg=c_implementation_active");
            println!("cargo:rustc-env=C_COMPILER_NAME={}", compiler_name);
        }
    }
}
