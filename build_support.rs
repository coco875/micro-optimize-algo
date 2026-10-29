//! Pure discovery/code generation, also exercised against temporary source trees.
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[path = "macros/src/metadata.rs"]
mod metadata;

struct Source {
    path: PathBuf,
    module: String,
    file: syn::File,
}
struct Family {
    group: String,
    reference: syn::Path,
    path: PathBuf,
    cfg: Vec<syn::Attribute>,
}
struct Entry {
    group: String,
    id: String,
    name: String,
    path: syn::Path,
    cfg: Vec<syn::Attribute>,
    tags: Vec<String>,
}

fn cfgs(attrs: &[syn::Attribute]) -> Vec<syn::Attribute> {
    attrs
        .iter()
        .filter_map(|attribute| {
            let meta = cfg_meta(&attribute.meta)?;
            let mut attr = attribute.clone();
            attr.meta = meta;
            attr.style = syn::AttrStyle::Outer;
            Some(attr)
        })
        .collect()
}

// Copy only conditional-compilation effects, not e.g. conditional derives or docs.
fn cfg_meta(meta: &syn::Meta) -> Option<syn::Meta> {
    if meta.path().is_ident("cfg") {
        return Some(meta.clone());
    }
    let syn::Meta::List(list) = meta else {
        return None;
    };
    if !list.path.is_ident("cfg_attr") {
        return None;
    }
    let args = list
        .parse_args_with(syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
        .ok()?;
    let condition = args.first()?;
    let effects: Vec<_> = args.iter().skip(1).filter_map(cfg_meta).collect();
    if effects.is_empty() {
        return None;
    }
    Some(syn::parse_quote!(cfg_attr(#condition, #(#effects),*)))
}
fn annotation(
    attrs: &[syn::Attribute],
    kind: &str,
) -> Result<Option<BTreeMap<String, String>>, String> {
    let mut found = None;
    for attr in attrs {
        if attr.path().segments.last().is_some_and(|s| s.ident == kind) {
            if found.is_some() {
                return Err(format!("duplicate {kind} annotation"));
            }
            let syn::Meta::List(list) = &attr.meta else {
                return Err(format!("invalid {kind} annotation"));
            };
            found = Some(metadata::parse(list.tokens.clone(), kind).map_err(|e| e.to_string())?);
        }
    }
    Ok(found)
}
fn write(out: &Path, name: &str, tokens: TokenStream) -> Result<(), String> {
    let path = out.join(name);
    fs::write(&path, tokens.to_string()).map_err(|e| e.to_string())?;
    // Formatting generated artifacts keeps rustc diagnostics readable.
    let _ = std::process::Command::new("rustfmt")
        .arg("--edition=2021")
        .args(["--config", "skip_children=true"])
        .arg(&path)
        .output();
    Ok(())
}

pub fn generate(src: &Path, out: &Path) -> Result<(), String> {
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let mut paths = Vec::new();
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "rs") && entry.file_type().is_file() {
            let relative = path.strip_prefix(src).unwrap();
            if relative.starts_with("bin") || relative == Path::new("registry.rs") {
                continue;
            }
            paths.push(path.to_owned());
        }
    }
    paths.sort();
    let mut sources = Vec::new();
    for path in paths {
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let file = syn::parse_file(&content).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut components: Vec<_> = path
            .strip_prefix(src)
            .unwrap()
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        let last = components.pop().unwrap();
        if last != "mod.rs" && last != "lib.rs" {
            components.push(last.trim_end_matches(".rs").into());
        }
        sources.push(Source {
            path,
            module: format!("crate::{}", components.join("::")),
            file,
        });
    }
    let mut families = BTreeMap::new();
    let mut family_paths = BTreeSet::new();
    for source in &sources {
        for item in &source.file.items {
            let attrs = match item {
                syn::Item::Struct(i) => &i.attrs,
                syn::Item::Trait(i) => &i.attrs,
                syn::Item::Fn(i) => &i.attrs,
                syn::Item::Enum(i) => &i.attrs,
                syn::Item::Type(i) => &i.attrs,
                syn::Item::Mod(i) => &i.attrs,
                syn::Item::Impl(i) => &i.attrs,
                syn::Item::Const(i) => &i.attrs,
                syn::Item::Static(i) => &i.attrs,
                syn::Item::Union(i) => &i.attrs,
                _ => continue,
            };
            for kind in ["algorithm", "variant", "scenario"] {
                if annotation(attrs, kind)?.is_some() {
                    let valid = if kind == "algorithm" {
                        matches!(item, syn::Item::Trait(_))
                    } else {
                        matches!(item, syn::Item::Struct(_))
                    };
                    if !valid {
                        return Err(format!(
                            "{}: {kind} annotation is on the wrong item kind",
                            source.path.display()
                        ));
                    }
                }
            }
            if let syn::Item::Trait(item) = item {
                if let Some(meta) = annotation(&item.attrs, "algorithm")? {
                    if !family_paths.insert(source.path.clone()) {
                        return Err(
                            "only one algorithm contract is allowed per family directory".into(),
                        );
                    }
                    let relative = source.path.strip_prefix(src).unwrap();
                    if relative.components().count() != 3
                        || source.path.file_name().unwrap() != "mod.rs"
                    {
                        return Err(
                            "algorithm contracts must live in src/<category>/<family>/mod.rs"
                                .into(),
                        );
                    }
                    let reference = format!("{}::{}", source.module, meta["reference"]);
                    let group = meta["group"].clone();
                    let mut cfg = cfgs(&source.file.attrs);
                    cfg.extend(cfgs(&item.attrs));
                    let family = Family {
                        group: group.clone(),
                        reference: syn::parse_str(&reference).map_err(|e| e.to_string())?,
                        path: source.path.clone(),
                        cfg,
                    };
                    if families.insert(group.clone(), family).is_some() {
                        return Err(format!("duplicate family ID: {group}"));
                    }
                }
            }
        }
    }
    let mut scenarios = Vec::new();
    let mut variants = Vec::new();
    let mut ids = BTreeSet::new();
    for source in &sources {
        for item in &source.file.items {
            if let syn::Item::Struct(item) = item {
                for kind in ["variant", "scenario"] {
                    if let Some(meta) = annotation(&item.attrs, kind)? {
                        let group = meta["group"].clone();
                        let family = families.get(&group).ok_or_else(|| {
                            format!("unknown family {group} in {}", source.path.display())
                        })?;
                        let directory = family.path.parent().unwrap();
                        let expected = directory.join(if kind == "variant" {
                            "code"
                        } else {
                            "scenarios"
                        });
                        if source.path.parent() != Some(expected.as_path()) {
                            return Err(format!(
                                "{kind} must live directly under {}",
                                expected.display()
                            ));
                        }
                        let id = meta
                            .get("id")
                            .cloned()
                            .unwrap_or_else(|| item.ident.to_string());
                        if !ids.insert((kind, group.clone(), id.clone())) {
                            return Err(format!("duplicate {kind} ID: {group}/{id}"));
                        }
                        let mut cfg = family.cfg.clone();
                        // code/mod.rs can gate all its children.
                        if let Some(parent) =
                            sources.iter().find(|s| s.path == expected.join("mod.rs"))
                        {
                            cfg.extend(cfgs(&parent.file.attrs));
                        }
                        cfg.extend(cfgs(&source.file.attrs));
                        cfg.extend(cfgs(&item.attrs));
                        let entry = Entry {
                            group,
                            id,
                            name: meta.get("name").cloned().unwrap_or_else(|| {
                                item.ident.to_string().trim_end_matches("Scenario").into()
                            }),
                            path: syn::parse_str(&format!("{}::{}", source.module, item.ident))
                                .map_err(|e| e.to_string())?,
                            cfg,
                            tags: meta
                                .get("tags")
                                .map(|s| {
                                    s.split(',')
                                        .map(str::trim)
                                        .filter(|s| !s.is_empty())
                                        .map(str::to_owned)
                                        .collect()
                                })
                                .unwrap_or_default(),
                        };
                        if kind == "variant" {
                            variants.push(entry);
                        } else {
                            scenarios.push(entry);
                        }
                    }
                }
            }
        }
    }
    variants.sort_by(|a, b| (&a.group, &a.id).cmp(&(&b.group, &b.id)));
    scenarios.sort_by(|a, b| (&a.group, &a.id).cmp(&(&b.group, &b.id)));
    for family in families.values() {
        let reference = &family.reference;
        if !variants.iter().any(|v| {
            quote!(#reference).to_string() == {
                let p = &v.path;
                quote!(#p).to_string()
            } && v.group == family.group
        }) {
            return Err(format!("unknown reference for {}", family.group));
        }
        if !scenarios.iter().any(|s| s.group == family.group) {
            return Err(format!("family {} has no scenarios", family.group));
        }
    }

    let mut categories: BTreeMap<String, Vec<TokenStream>> = BTreeMap::new();
    for family in families.values() {
        let relative = family.path.strip_prefix(src).unwrap();
        let parts: Vec<_> = relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        let ident = format_ident!("{}", parts[1]);
        let path = family.path.to_string_lossy().to_string();
        let cfg = &family.cfg;
        categories
            .entry(parts[0].clone())
            .or_default()
            .push(quote!(#(#cfg)* #[path = #path] pub mod #ident;));
        for (folder, prefix) in [("code", "implementations"), ("scenarios", "scenarios")] {
            let directory = family.path.parent().unwrap().join(folder);
            let mut modules = Vec::new();
            for source in sources.iter().filter(|s| {
                s.path.parent() == Some(directory.as_path())
                    && s.path.file_name().unwrap() != "mod.rs"
            }) {
                let ident = format_ident!("{}", source.path.file_stem().unwrap().to_string_lossy());
                let path = source.path.to_string_lossy().to_string();
                let cfg = cfgs(&source.file.attrs);
                modules.push(quote!(#(#cfg)* #[path = #path] pub mod #ident;));
            }
            let tokens = if folder == "scenarios" {
                let code_module = family.path.parent().unwrap().join("code/mod.rs");
                let code = if code_module.exists() {
                    let cfg = cfgs(
                        &sources
                            .iter()
                            .find(|s| s.path == code_module)
                            .unwrap()
                            .file
                            .attrs,
                    );
                    let path = code_module.to_string_lossy().to_string();
                    quote!(#(#cfg)* #[path = #path] pub mod code;)
                } else {
                    let key = format!("{}/{}", parts[0], parts[1]);
                    quote!(
                        pub mod code {
                            crate::implementation_modules!(#key);
                        }
                    )
                };
                quote!(#code pub mod scenarios { #(#modules)* })
            } else {
                quote!(#(#modules)*)
            };
            write(
                out,
                &format!("{prefix}_{}_{}.rs", parts[0], parts[1]),
                tokens,
            )?;
        }
    }
    let root: Vec<_> = categories
        .into_iter()
        .map(|(name, modules)| {
            let name = format_ident!("{name}");
            quote!(pub mod #name { #(#modules)* })
        })
        .collect();
    write(out, "algorithms.rs", quote!(#(#root)*))?;

    let mut fields = Vec::new();
    let mut initialization = Vec::new();
    let mut field_values = Vec::new();
    let mut schedules = Vec::new();
    let mut run_arms = Vec::new();
    let mut metadata_arms = Vec::new();
    let mut verify_functions = Vec::new();
    let mut verify_calls = Vec::new();
    let mut tests = Vec::new();
    for (s_idx, scenario) in scenarios.iter().enumerate() {
        let s_idx = u16::try_from(s_idx).map_err(|_| "too many scenarios")?;
        let cfg = &scenario.cfg;
        let family = &families[&scenario.group];
        let reference = &family.reference;
        let path = &scenario.path;
        let sc_ref = quote!(#path::<#reference>);
        let field = format_ident!("scenario_{s_idx}_params");
        let group = &scenario.group;
        let id = &scenario.id;
        let name = &scenario.name;
        let tags = &scenario.tags;
        fields.push(
            quote!(#(#cfg)* pub #field: Vec<<#sc_ref as ScenarioDefinition<#reference>>::Param>),
        );
        initialization.push(quote! {
            #(#cfg)* let #field = #sc_ref::generate_params(config);
            #(#cfg)* {
                let mut labels = std::collections::BTreeSet::new();
                for param in &#field { assert!(labels.insert(#sc_ref::format_param(param)), "duplicate benchmark parameter label"); }
            }
        });
        field_values.push(quote!(#(#cfg)* #field));
        let mut variant_schedules = Vec::new();
        let mut variant_runs = Vec::new();
        let mut variant_metadata = Vec::new();
        let mut variant_validation = Vec::new();
        let sc_variants: Vec<_> = variants.iter().filter(|v| v.group == *group).collect();
        for (v_idx, variant) in sc_variants.iter().enumerate() {
            let v_idx = u16::try_from(v_idx).map_err(|_| "too many variants")?;
            let v_cfg = &variant.cfg;
            let v_path = &variant.path;
            let v_name = &variant.id;
            let sc_var = quote!(#path::<#v_path>);
            variant_schedules.push(quote! {
                #(#v_cfg)* if config.matches_variant(#v_name) {
                    for (param_idx, param) in #field.iter().enumerate() {
                        let label = #sc_ref::format_param(param);
                        let seed = derive_seed(root_seed, &["input", #group, #id, &label, &sample.to_string()]);
                        run_ids.push(RunId::new(#s_idx, #v_idx, u32::try_from(param_idx).expect("too many parameters"), seed));
                    }
                }
            });
            variant_runs.push(quote! {
                #(#v_cfg)* #v_idx => {
                    let fixture = #sc_ref::prepare(param, &mut InputRng::new(seed));
                    let mut instance = #sc_var::new(&fixture);
                    let timer = crate::utils::scenario::CpuTimer::new();
                    instance.task();
                    timer.elapsed_cycles()
                }
            });
            variant_metadata.push(quote! {
                #(#v_cfg)* #v_idx => RunMetadata { group: #group, scenario: #name, scenario_id: #id, variant: #v_name,
                    context: #sc_ref::format_param(param), seed, tags: &[#(#tags),*] },
            });
            let is_reference = quote!(#reference).to_string() == quote!(#v_path).to_string();
            let actual = if is_reference {
                quote!(let actual = reference_output.as_ref().expect("reference output");)
            } else {
                quote!(let actual_output = #sc_var::validate(&fixture); let actual = &actual_output;)
            };
            variant_validation.push(quote! {
                #(#v_cfg)* if config.matches_variant(#v_name) {
                    #actual
                    record_checks(report, (#group, #name, #v_name, &case.id, seed), &checks, actual, reference_output.as_ref());
                }
            });
        }
        schedules.push(quote! { #(#cfg)* if config.matches_scenario(#group, #id, #name, &[#(#tags),*]) { #(#variant_schedules)* } });
        run_arms.push(quote! { #(#cfg)* #s_idx => { let param = &session.#field[param_idx as usize]; match variant_idx { #(#variant_runs)* _ => panic!("invalid variant index") } } });
        metadata_arms.push(quote! { #(#cfg)* #s_idx => { let param = &session.#field[param_idx as usize]; match variant_idx { #(#variant_metadata)* _ => panic!("invalid variant index") } } });
        let reference_id = &sc_variants
            .iter()
            .find(|v| {
                let p = &v.path;
                quote!(#p).to_string() == quote!(#reference).to_string()
            })
            .unwrap()
            .id;
        let enabled_checks: Vec<_> = sc_variants
            .iter()
            .map(|v| {
                let cfg = &v.cfg;
                let id = &v.id;
                quote!(#(#cfg)* { selected |= config.matches_variant(#id); })
            })
            .collect();
        let verify_fn = format_ident!("verify_{s_idx}");
        verify_functions.push(quote! {
            #(#cfg)* fn #verify_fn(config: &Config, root_seed: u64, report: &mut ValidationReport) {
                let mut selected = false; #(#enabled_checks)*
                if !selected { return; }
                let mut case_ids = std::collections::BTreeSet::new();
                let cases = #sc_ref::validation_cases(config);
                assert!(!cases.is_empty(), "scenario has no validation cases");
                for case in cases {
                    assert!(case_ids.insert(case.id.clone()), "duplicate validation case ID");
                    let seed = derive_seed(root_seed, &["validation", #group, #id, &case.id]);
                    let fixture = #sc_ref::prepare(&case.param, &mut InputRng::new(seed));
                    let checks = #sc_ref::checks(&fixture);
                    let reference_output = if checks.iter().any(|c| c.needs_reference()) || config.matches_variant(#reference_id) {
                        Some(#sc_ref::validate(&fixture))
                    } else { None };
                    #(#variant_validation)*
                }
            }
        });
        verify_calls.push(quote! { #(#cfg)* if config.matches_scenario(#group, #id, #name, &[#(#tags),*]) { #verify_fn(config, root_seed, &mut report); } });
        let test_fn = format_ident!("validation_{s_idx}");
        tests.push(quote! { #(#cfg)* #[test] fn #test_fn() {
            let mut report = ValidationReport::default();
            #verify_fn(&Config::default(), crate::utils::randomness::VALIDATION_SEED, &mut report);
            if !report.passed() { report.print(); panic!("scenario validation failed"); }
        } });
    }
    write(
        out,
        "registry.rs",
        quote! {
            use crate::utils::runner::{Config, RunId, RunMetadata};
            use crate::utils::scenario::{Scenario, ScenarioDefinition};
            use crate::utils::randomness::{derive_seed, InputRng};
            use crate::utils::validation::{record_checks, ValidationReport};
            pub struct BenchmarkSession { pub root_seed: u64, pub run_ids: Vec<RunId>, #(#fields,)* }
            pub fn init_session(config: &Config) -> BenchmarkSession {
            let root_seed = config.seed.unwrap_or_else(crate::utils::randomness::fresh_seed);
                let mut run_ids = Vec::new(); #(#initialization)*
                for sample in 0..config.sample.unwrap_or(10) { #(#schedules)* }
                InputRng::new(derive_seed(root_seed, &["schedule"])).shuffle(&mut run_ids);
                BenchmarkSession { root_seed, run_ids, #(#field_values,)* }
            }
            pub fn run_scenario(session: &BenchmarkSession, run_id: RunId) -> u64 {
                let (scenario_idx, variant_idx, param_idx, seed) = run_id.unpack();
                match scenario_idx { #(#run_arms)* _ => panic!("invalid scenario index") }
            }
            pub fn get_run_metadata(session: &BenchmarkSession, run_id: RunId) -> RunMetadata {
                let (scenario_idx, variant_idx, param_idx, seed) = run_id.unpack();
                match scenario_idx { #(#metadata_arms)* _ => panic!("invalid scenario index") }
            }
            #(#verify_functions)*
            pub fn verify(config: &Config) -> ValidationReport {
                let root_seed = config.seed.unwrap_or(crate::utils::randomness::VALIDATION_SEED);
                let mut report = ValidationReport::default(); #(#verify_calls)* report
            }
            #[cfg(test)] mod generated_validation { use super::*; #(#tests)* }
        },
    )
}
