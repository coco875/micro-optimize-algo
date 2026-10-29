#[path = "../build_support.rs"]
mod build_support;

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Tree(PathBuf);
impl Tree {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "micro-algo-discovery-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn put(&self, path: &str, contents: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
    fn generate(&self) -> Result<(), String> {
        build_support::generate(&self.0.join("src"), &self.0.join("out"))
    }
    fn read(&self, file: &str) -> String {
        fs::read_to_string(self.0.join("out").join(file)).unwrap()
    }
}
impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const CONTRACT: &str = r#"
#[crate::algorithm(group = "counter", reference = "code::original::Original")]
pub trait System: Send + 'static { fn compute(value: u64) -> u64; }
crate::algorithm_modules!("general/counter");
"#;
const REFERENCE: &str = r#"
use std::sync::atomic::{AtomicUsize, Ordering};
pub static CALLS: AtomicUsize = AtomicUsize::new(0);
#[crate::variant(group = "counter")]
pub struct Original;
impl super::super::System for Original {
    fn compute(value: u64) -> u64 { CALLS.fetch_add(1, Ordering::SeqCst); value.wrapping_add(1) }
}
"#;
const VARIANT: &str = r#"
use std::sync::atomic::{AtomicUsize, Ordering};
pub static CALLS: AtomicUsize = AtomicUsize::new(0);
#[crate::variant(group = "counter")]
pub struct Fast;
impl super::super::System for Fast {
    fn compute(value: u64) -> u64 { CALLS.fetch_add(1, Ordering::SeqCst); value.wrapping_add(1) }
}
"#;
const SCENARIO: &str = r#"
use crate::utils::{runner::Config, randomness::InputRng, scenario::{Scenario, ScenarioDefinition, ValidationCase}, validation::{Check, Exact}};
use super::super::System;
use std::sync::atomic::{AtomicUsize, Ordering};
pub static VALIDATIONS: AtomicUsize = AtomicUsize::new(0);
#[crate::scenario(group = "counter", id = "single", name = "Single", tags = "general")]
pub struct Single<V> { state: u64, marker: std::marker::PhantomData<V> }
impl<V: System> Scenario for Single<V> {
    fn task(&mut self) { self.state = std::hint::black_box(V::compute(self.state)); }
}
impl<V: System> ScenarioDefinition<V> for Single<V> {
    type Param = usize;
    type Fixture = u64;
    type Output = u64;
    fn prepare(_: &usize, rng: &mut InputRng) -> u64 { rng.next_u64() }
    fn new(f: &u64) -> Self { Self { state: *f, marker: std::marker::PhantomData } }
    fn generate_params(_: &Config) -> Vec<usize> { vec![1] }
    fn format_param(p: &usize) -> String { p.to_string() }
    fn validation_cases(_: &Config) -> Vec<ValidationCase<usize>> { vec![ValidationCase::new("single", 1)] }
    fn validate(f: &u64) -> u64 { VALIDATIONS.fetch_add(1, Ordering::SeqCst); V::compute(*f) }
    fn checks(_: &u64) -> Vec<Box<dyn Check<u64>>> { vec![Box::new(Exact), Box::new(Exact)] }
}
pub fn assert_fresh_state() {
    let mut a = Single::<super::super::code::original::Original>::new(&42);
    let mut b = Single::<super::super::code::original::Original>::new(&42);
    a.task(); a.task(); b.task();
    assert_eq!(a.state, 44); assert_eq!(b.state, 43);
}
"#;

fn fixture() -> Tree {
    let tree = Tree::new();
    tree.put("src/general/counter/mod.rs", CONTRACT);
    tree.put("src/general/counter/code/original.rs", REFERENCE);
    tree.put("src/general/counter/scenarios/single.rs", SCENARIO);
    tree
}

#[test]
fn single_file_additions_deletions_and_cfg_are_discovered() {
    let tree = fixture();
    tree.generate().unwrap();
    let before = tree.read("registry.rs");
    tree.put("src/general/counter/code/fast.rs", VARIANT);
    tree.put(
        "src/general/counter/code/optional.rs",
        &format!("#![cfg(any())]\n{}", VARIANT.replace("Fast", "Optional")),
    );
    tree.put(
        "src/general/counter/scenarios/second.rs",
        &SCENARIO
            .replace("Single", "Second")
            .replace("single", "second"),
    );
    tree.generate().unwrap();
    let after = tree.read("registry.rs");
    assert!(after.contains("Fast") && !before.contains("Fast"));
    assert!(after.contains("Second"));
    assert!(tree
        .read("implementations_general_counter.rs")
        .contains("#[cfg(any())]"));
    syn::parse_file(&after).unwrap();
    fs::remove_file(tree.0.join("src/general/counter/code/fast.rs")).unwrap();
    tree.generate().unwrap();
    assert!(!tree.read("registry.rs").contains("Fast"));
    assert_eq!(
        fs::read_to_string(tree.0.join("src/general/counter/mod.rs")).unwrap(),
        CONTRACT
    );
}

#[test]
fn invalid_registration_is_rejected() {
    let tree = fixture();
    tree.put("src/general/counter/code/duplicate.rs", REFERENCE);
    assert!(tree
        .generate()
        .unwrap_err()
        .contains("duplicate variant ID"));
    fs::remove_file(tree.0.join("src/general/counter/code/duplicate.rs")).unwrap();
    tree.put(
        "src/general/counter/code/fast.rs",
        &VARIANT.replace("counter", "unknown"),
    );
    assert!(tree.generate().unwrap_err().contains("unknown family"));
    tree.put(
        "src/general/counter/code/fast.rs",
        &VARIANT.replace("group =", "typo ="),
    );
    assert!(tree
        .generate()
        .unwrap_err()
        .contains("unknown annotation key"));
    fs::remove_file(tree.0.join("src/general/counter/code/fast.rs")).unwrap();
    tree.put(
        "src/general/counter/mod.rs",
        &CONTRACT.replace("code::original::Original", "code::original::Missing"),
    );
    assert!(tree.generate().unwrap_err().contains("unknown reference"));
    tree.put(
        "src/general/counter/mod.rs",
        "#[crate::algorithm(group = \"counter\", reference = \"code::original::Original\")] pub struct System;",
    );
    assert!(tree.generate().unwrap_err().contains("wrong item kind"));
    tree.put("src/general/counter/mod.rs", CONTRACT);
    tree.put("src/general/counter/code/fast.rs", "this is not valid Rust");
    assert!(tree.generate().unwrap_err().contains("fast.rs"));
}

fn library(deps: &Path) -> PathBuf {
    let mut candidates: Vec<_> = fs::read_dir(deps)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("libmicro_optimize_algo-")
                && p.extension().is_some_and(|e| e == "rlib")
        })
        .collect();
    candidates.sort_by_key(|p| fs::metadata(p).unwrap().modified().unwrap());
    candidates.pop().expect("compiled library")
}

#[test]
fn generated_dispatch_reuses_reference_and_never_validates_benchmarks() {
    let tree = fixture();
    tree.put("src/general/counter/code/fast.rs", VARIANT);
    tree.put(
        "src/general/counter/code/broken.rs",
        &VARIANT
            .replace("Fast", "Broken")
            .replace("value.wrapping_add(1)", "value"),
    );
    let quality = SCENARIO
        .replace("Single", "Quality")
        .replace("single", "quality")
        .replace("fn checks(_: &u64)", "fn checks(fixture: &u64)")
        .replace(
            "vec![Box::new(Exact), Box::new(Exact)]",
            "vec![Box::new(Expected(fixture.wrapping_add(1)))]",
        );
    tree.put("src/general/counter/scenarios/quality.rs", &(quality + r#"
pub struct Expected(u64);
impl Check<u64> for Expected {
    fn name(&self) -> &str { "expected" }
    fn needs_reference(&self) -> bool { false }
    fn evaluate(&self, value: &u64, _: Option<&u64>) -> crate::utils::validation::CheckResult {
        crate::utils::validation::CheckResult::new(*value == self.0, value.to_string(), self.0.to_string(), "independent oracle")
    }
}
"#));
    // A disabled implementation contains deliberately invalid Rust types. It must disappear everywhere.
    tree.put(
        "src/general/counter/code/optional.rs",
        "#![cfg_attr(all(), cfg(any()))]\n#[crate::variant(group = \"counter\")] pub struct Optional(UnknownType);",
    );
    tree.put("src/general/counter/scenarios/disabled.rs", "#![cfg(any())]\n#[crate::scenario(group = \"counter\")] pub struct Disabled<V>(UnknownType, V);");
    tree.generate().unwrap();
    tree.put("harness.rs", r#"
pub use micro_optimize_algo::{algorithm, variant, scenario, algorithm_modules, implementation_modules};
pub mod utils { pub use micro_optimize_algo::utils::{runner, scenario, validation, randomness}; }
include!(concat!(env!("OUT_DIR"), "/algorithms.rs"));
pub mod registry { include!(concat!(env!("OUT_DIR"), "/registry.rs")); }
fn main() {
    use std::sync::atomic::Ordering::SeqCst;
    use general::counter::{code::{original, fast}, scenarios::single};
    let config = utils::runner::Config { sample: Some(3), seed: Some(42), filter_scenario: Some(vec!["single".into()]), filter_variant: Some(vec!["Original".into(), "Fast".into()]), ..Default::default() };
    let report = registry::verify(&config);
    assert!(report.passed()); assert_eq!(report.records.len(), 4);
    assert_eq!(original::CALLS.load(SeqCst), 1); assert_eq!(fast::CALLS.load(SeqCst), 1);
    assert_eq!(single::VALIDATIONS.load(SeqCst), 2);
    let session = registry::init_session(&config);
    let replay = registry::init_session(&config);
    assert_eq!(session.run_ids, replay.run_ids);
    for &run in &session.run_ids { registry::run_scenario(&session, run); }
    assert_eq!(single::VALIDATIONS.load(SeqCst), 2);
    assert_eq!(original::CALLS.load(SeqCst), 4); assert_eq!(fast::CALLS.load(SeqCst), 4);
    single::assert_fresh_state();
    let filtered = utils::runner::Config { filter_variant: Some(vec!["Fast".into()]), ..config.clone() };
    let selected = registry::init_session(&filtered);
    let seeds = |s: &registry::BenchmarkSession| {
        let mut seeds: Vec<_> = s.run_ids.iter().filter_map(|&r| { let m = registry::get_run_metadata(s,r); (m.variant == "Fast").then_some(m.seed) }).collect();
        seeds.sort(); seeds
    };
    assert_eq!(seeds(&session), seeds(&selected));
    let filtered_report = registry::verify(&filtered);
    assert!(filtered_report.passed()); assert_eq!(filtered_report.records.len(), 2);
    let reference_calls = original::CALLS.load(SeqCst);
    let quality = registry::verify(&utils::runner::Config { filter_scenario: Some(vec!["quality".into()]), ..filtered.clone() });
    assert!(quality.passed()); assert_eq!(quality.records.len(), 1);
    assert_eq!(original::CALLS.load(SeqCst), reference_calls);
    let broken = registry::verify(&utils::runner::Config { filter_variant: Some(vec!["Broken".into()]), ..config });
    assert!(!broken.passed()); assert_eq!(broken.records.len(), 2);
    assert_eq!(general::counter::code::broken::CALLS.load(SeqCst), 1);
}
"#);
    let deps = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let binary = tree.0.join(if cfg!(windows) {
        "harness.exe"
    } else {
        "harness"
    });
    let output = Command::new("rustc")
        .args(["--edition=2021", "--crate-name=discovery_harness"])
        .arg(tree.0.join("harness.rs"))
        .arg("--extern")
        .arg(format!("micro_optimize_algo={}", library(&deps).display()))
        .arg("-L")
        .arg(format!("dependency={}", deps.display()))
        .arg("-o")
        .arg(&binary)
        .env("OUT_DIR", tree.0.join("out"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new(binary).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
