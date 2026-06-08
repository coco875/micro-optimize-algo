//! Build script to compile C implementations and generate registry.rs.

use std::env;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    // 1. Generate src/registry.rs
    generate_registry();

    // 2. Compile C Implementations
    compile_c_code();
}

fn get_scenario_attrs(attr: &syn::Attribute) -> Option<(String, Option<String>, Option<String>)> {
    if attr.path().is_ident("scenario") {
        if let syn::Meta::List(ref meta_list) = attr.meta {
            let nested = meta_list
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::token::Comma>::parse_terminated,
                )
                .ok()?;
            let mut group = None;
            let mut name = None;
            let mut tags = None;
            for meta in nested {
                if let syn::Meta::NameValue(ref nv) = meta {
                    if nv.path.is_ident("group") {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(ref s),
                            ..
                        }) = nv.value
                        {
                            group = Some(s.value());
                        }
                    } else if nv.path.is_ident("name") {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(ref s),
                            ..
                        }) = nv.value
                        {
                            name = Some(s.value());
                        }
                    } else if nv.path.is_ident("tags") {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(ref s),
                            ..
                        }) = nv.value
                        {
                            tags = Some(s.value());
                        }
                    }
                }
            }
            if let Some(g) = group {
                return Some((g, name, tags));
            }
        }
    }
    None
}

fn get_variant_group(attr: &syn::Attribute) -> Option<String> {
    if attr.path().is_ident("variant") {
        if let syn::Meta::List(ref meta_list) = attr.meta {
            let nested = meta_list
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::token::Comma>::parse_terminated,
                )
                .ok()?;
            for meta in nested {
                if let syn::Meta::NameValue(ref nv) = meta {
                    if nv.path.is_ident("group") {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(ref s),
                            ..
                        }) = nv.value
                        {
                            return Some(s.value());
                        }
                    }
                }
            }
        }
    }
    None
}

fn get_cfgs_as_strings(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("cfg"))
        .map(|a| quote::quote!(#a).to_string())
        .collect()
}

fn generate_registry() {
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = Path::new(&out_dir).join("src");
    let dest_path = src_dir.join("registry.rs");

    struct ScenarioInfo {
        struct_name: String,
        group: String,
        scenario_name: String,
        module_path: String,
        tags: Vec<String>,
    }
    struct VariantInfo {
        struct_name: String,
        group: String,
        module_path: String,
        cfgs: Vec<String>,
    }
    struct ImplInfo {
        struct_name: String,
        param_type: String,
    }

    let mut scenarios = Vec::new();
    let mut variants = Vec::new();
    let mut impls = Vec::new();

    for entry in WalkDir::new(&src_dir) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            if path == dest_path {
                continue;
            }
            if path.components().any(|c| c.as_os_str() == "bin") {
                continue;
            }

            let content = fs::read_to_string(path).unwrap();
            let file = match syn::parse_file(&content) {
                Ok(f) => f,
                Err(_) => continue,
            };

            let relative = path.strip_prefix(&src_dir).unwrap();
            let mut components: Vec<_> = relative
                .components()
                .map(|c| c.as_os_str().to_str().unwrap().to_string())
                .collect();
            if let Some(last) = components.last_mut() {
                if last == "mod.rs" || last == "lib.rs" {
                    components.pop();
                } else if last.ends_with(".rs") {
                    *last = last.strip_suffix(".rs").unwrap().to_string();
                }
            }
            let module_path = if components.is_empty() {
                "crate".to_string()
            } else {
                format!("crate::{}", components.join("::"))
            };

            for item in &file.items {
                match item {
                    syn::Item::Struct(syn_struct) => {
                        for attr in &syn_struct.attrs {
                            if let Some((group, name, tags)) = get_scenario_attrs(attr) {
                                let scenario_name = name.unwrap_or_else(|| {
                                    let s = syn_struct.ident.to_string();
                                    s.strip_suffix("Scenario").unwrap_or(&s).to_string()
                                });
                                let tags_list = tags
                                    .map(|t| {
                                        t.split(',')
                                            .map(|s| s.trim().to_string())
                                            .filter(|s| !s.is_empty())
                                            .collect::<Vec<String>>()
                                    })
                                    .unwrap_or_default();
                                scenarios.push(ScenarioInfo {
                                    struct_name: syn_struct.ident.to_string(),
                                    group,
                                    scenario_name,
                                    module_path: module_path.clone(),
                                    tags: tags_list,
                                });
                            }
                            if let Some(group) = get_variant_group(attr) {
                                let cfgs = get_cfgs_as_strings(&syn_struct.attrs);
                                variants.push(VariantInfo {
                                    struct_name: syn_struct.ident.to_string(),
                                    group,
                                    module_path: module_path.clone(),
                                    cfgs,
                                });
                            }
                        }
                    }
                    syn::Item::Impl(item_impl) => {
                        if let Some((_, ref trait_path, _)) = item_impl.trait_ {
                            if trait_path
                                .segments
                                .last()
                                .is_some_and(|s| s.ident == "ScenarioDefinition")
                            {
                                if let syn::Type::Path(ref type_path) = *item_impl.self_ty {
                                    if let Some(segment) = type_path.path.segments.first() {
                                        let struct_name = segment.ident.to_string();
                                        for impl_item in &item_impl.items {
                                            if let syn::ImplItem::Type(ref ty) = impl_item {
                                                if ty.ident == "Param" {
                                                    let param_ty = &ty.ty;
                                                    impls.push(ImplInfo {
                                                        struct_name: struct_name.clone(),
                                                        param_type: quote::quote!(#param_ty)
                                                            .to_string(),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let mut code = String::new();
    code.push_str("//! Generated by build.rs. Do not edit manually.\n\n");
    code.push_str(
        "#![allow(unused_variables, dead_code, unused_mut, unused_imports, clippy::all)]\n\n",
    );
    code.push_str("use crate::utils::runner::{Config, RunId, RunMetadata};\n");
    code.push_str("use crate::utils::Scenario;\n");
    code.push_str("use crate::utils::scenario::ScenarioDefinition;\n");
    code.push_str("use rand::prelude::*;\n\n");

    code.push_str("pub struct BenchmarkSession {\n");
    code.push_str("    pub run_ids: Vec<RunId>,\n");
    for (idx, sc) in scenarios.iter().enumerate() {
        let param_type = impls
            .iter()
            .find(|i| i.struct_name == sc.struct_name)
            .map(|i| i.param_type.as_str())
            .unwrap_or("usize");
        code.push_str(&format!(
            "    pub scenario_{}_params: Vec<{}>,\n",
            idx, param_type
        ));
    }
    code.push_str("}\n\n");

    code.push_str("pub fn init_session(config: &Config) -> BenchmarkSession {\n");
    code.push_str("    let mut run_ids = Vec::new();\n");
    code.push_str("    let mut rng = rand::rngs::SmallRng::seed_from_u64(0);\n");
    code.push_str("    let sample = config.sample.unwrap_or(10);\n\n");

    for (idx, sc) in scenarios.iter().enumerate() {
        let first_variant = variants
            .iter()
            .find(|v| v.group == sc.group && v.cfgs.is_empty())
            .or_else(|| variants.iter().find(|v| v.group == sc.group));
        if let Some(fv) = first_variant {
            let sc_path = format!("{}::{}", sc.module_path, sc.struct_name);
            let var_path = format!("{}::{}", fv.module_path, fv.struct_name);
            code.push_str(&format!(
                "    let scenario_{}_params = {}::<{}>::generate_params(config);\n",
                idx, sc_path, var_path
            ));
        } else {
            code.push_str(&format!(
                "    let scenario_{}_params: Vec<usize> = Vec::new();\n",
                idx
            ));
        }
    }
    code.push('\n');

    code.push_str("    for _ in 0..sample {\n");
    code.push_str("        let seed = rng.next_u64();\n");

    for (s_idx, sc) in scenarios.iter().enumerate() {
        let sc_variants: Vec<_> = variants.iter().filter(|v| v.group == sc.group).collect();
        if sc_variants.is_empty() {
            continue;
        }

        let tags_array = if sc.tags.is_empty() {
            "([] as &[&str])".to_string()
        } else {
            let tag_exprs: Vec<String> = sc.tags.iter().map(|t| format!("\"{}\"", t)).collect();
            format!("[{}]", tag_exprs.join(", "))
        };

        code.push_str(&format!(
            "        if config.filter_scenario.as_ref().map_or(true, |filters| {{\n\
             \t    filters.iter().any(|f| \"{}\".contains(f) || \"{}\".contains(f))\n\
             \t}}) && config.filter_tag.as_ref().map_or(true, |filters| {{\n\
             \t    filters.iter().any(|f| {}.contains(&f.as_str()))\n\
             \t}}) {{\n",
            sc.group, sc.scenario_name, tags_array
        ));

        for (v_idx, var) in sc_variants.iter().enumerate() {
            let cfgs_str = var.cfgs.join(" ");
            code.push_str(&format!(
                "            {} if config.filter_variant.as_ref().map_or(true, |filters| {{\n\
                 \t\tfilters.iter().any(|f| \"{}\".contains(f))\n\
                 \t\t}}) {{\n",
                cfgs_str, var.struct_name
            ));

            code.push_str(&format!(
                "                for param_idx in 0..scenario_{}_params.len() {{\n\
                 \t\t    run_ids.push(RunId::new({}, {}, param_idx as u32, seed));\n\
                 \t\t}}\n",
                s_idx, s_idx, v_idx
            ));
            code.push_str("            }\n");
        }
        code.push_str("        }\n");
    }

    code.push_str("    }\n\n");
    code.push_str("    run_ids.shuffle(&mut rng);\n\n");
    code.push_str("    BenchmarkSession {\n");
    code.push_str("        run_ids,\n");
    for (idx, _) in scenarios.iter().enumerate() {
        code.push_str(&format!("        scenario_{}_params,\n", idx));
    }
    code.push_str("    }\n");
    code.push_str("}\n\n");

    code.push_str("pub fn run_scenario(session: &BenchmarkSession, run_id: RunId) -> (u64, crate::utils::scenario::OtherMetric) {\n");
    code.push_str("    let (scenario_idx, variant_idx, param_idx, seed) = run_id.unpack();\n");
    code.push_str("    match scenario_idx {\n");

    for (s_idx, sc) in scenarios.iter().enumerate() {
        code.push_str(&format!("        {} => {{\n", s_idx));
        code.push_str(&format!(
            "            let param = session.scenario_{}_params[param_idx as usize].clone();\n",
            s_idx
        ));
        code.push_str("            match variant_idx {\n");

        let sc_variants: Vec<_> = variants.iter().filter(|v| v.group == sc.group).collect();
        for (v_idx, var) in sc_variants.iter().enumerate() {
            let cfgs_str = var.cfgs.join(" ");
            let sc_path = format!("{}::{}", sc.module_path, sc.struct_name);
            let var_path = format!("{}::{}", var.module_path, var.struct_name);

            code.push_str(&format!("                {} {} => {{\n", cfgs_str, v_idx));
            code.push_str(&format!(
                "                    let mut instance = {}::<{}>::new(param, seed);\n",
                sc_path, var_path
            ));
            code.push_str(
                "                    let timer = crate::utils::scenario::CpuTimer::new();\n",
            );
            code.push_str("                    instance.task();\n");
            code.push_str("                    let elapsed = timer.elapsed_cycles();\n");
            code.push_str("                    (elapsed, instance.other_metric())\n");
            code.push_str("                }\n");
        }

        code.push_str("                _ => panic!(\"Invalid variant index\"),\n");
        code.push_str("            }\n");
        code.push_str("        }\n");
    }

    code.push_str("        _ => panic!(\"Invalid scenario index\"),\n");
    code.push_str("    }\n");
    code.push_str("}\n\n");

    code.push_str(
        "pub fn get_run_metadata(session: &BenchmarkSession, run_id: RunId) -> RunMetadata {\n",
    );
    code.push_str("    let (scenario_idx, variant_idx, param_idx, seed) = run_id.unpack();\n");
    code.push_str("    match scenario_idx {\n");

    for (s_idx, sc) in scenarios.iter().enumerate() {
        code.push_str(&format!("        {} => {{\n", s_idx));
        code.push_str(&format!(
            "            let param = &session.scenario_{}_params[param_idx as usize];\n",
            s_idx
        ));
        code.push_str("            match variant_idx {\n");

        let sc_variants: Vec<_> = variants.iter().filter(|v| v.group == sc.group).collect();
        for (v_idx, var) in sc_variants.iter().enumerate() {
            let cfgs_str = var.cfgs.join(" ");
            let sc_path = format!("{}::{}", sc.module_path, sc.struct_name);
            let var_path = format!("{}::{}", var.module_path, var.struct_name);

            code.push_str(&format!("                {} {} => {{\n", cfgs_str, v_idx));
            code.push_str(&format!(
                "                    let context = {}::<{}>::format_param(param);\n",
                sc_path, var_path
            ));
            let tags_formatted = if sc.tags.is_empty() {
                "&[]".to_string()
            } else {
                let tag_exprs: Vec<String> = sc.tags.iter().map(|t| format!("\"{}\"", t)).collect();
                format!("&[{}]", tag_exprs.join(", "))
            };

            code.push_str(&format!(
                "                    RunMetadata {{\n\
                 \t\t\tgroup: \"{}\",\n\
                 \t\t\tscenario: \"{}\",\n\
                 \t\t\tvariant: \"{}\",\n\
                 \t\t\tcontext,\n\
                 \t\t\tseed,\n\
                 \t\t\ttags: {},\n\
                 \t\t\t}}\n",
                sc.group, sc.scenario_name, var.struct_name, tags_formatted
            ));
            code.push_str("                }\n");
        }

        code.push_str("                _ => panic!(\"Invalid variant index\"),\n");
        code.push_str("            }\n");
        code.push_str("        }\n");
    }

    code.push_str("        _ => panic!(\"Invalid scenario index\"),\n");
    code.push_str("    }\n");
    code.push_str("}\n");

    fs::write(&dest_path, code).unwrap();
    let _ = std::process::Command::new("rustfmt")
        .arg(&dest_path)
        .status();
    println!("cargo:rerun-if-changed={}", dest_path.display());
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
            build.opt_level(3).flag_if_supported("-ffast-math");

            if is_rust_native {
                build.flag_if_supported("-march=native");
            }

            build.compile("micro_algo_c");
            println!("cargo:rustc-cfg=c_implementation_active");
            println!("cargo:rustc-env=C_COMPILER_NAME={}", compiler_name);
        }
    }
}
