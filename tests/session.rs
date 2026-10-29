use micro_optimize_algo::{
    registry::{get_run_metadata, init_session, verify},
    utils::runner::Config,
};
use std::{collections::BTreeMap, process::Command};

#[test]
fn fixtures_survive_filters_and_share_seeds_across_implementations() {
    let config = Config {
        seed: Some(42),
        sample: Some(3),
        ..Default::default()
    };
    let session = init_session(&config);
    assert_eq!(session.run_ids, init_session(&config).run_ids);
    let mut groups: BTreeMap<_, BTreeMap<_, Vec<_>>> = BTreeMap::new();
    for &run in &session.run_ids {
        let m = get_run_metadata(&session, run);
        groups
            .entry((m.group, m.scenario_id, m.context))
            .or_default()
            .entry(m.variant)
            .or_default()
            .push(m.seed);
    }
    for variants in groups.values_mut() {
        for seeds in variants.values_mut() {
            seeds.sort();
        }
        let expected = variants.values().next().unwrap();
        assert_eq!(expected.len(), 3);
        for seeds in variants.values() {
            assert_eq!(seeds, expected);
        }
    }
    let filtered = init_session(&Config {
        filter_scenario: Some(vec!["dot_product".into()]),
        filter_variant: Some(vec!["Original".into()]),
        ..config.clone()
    });
    let collect = |session: &micro_optimize_algo::registry::BenchmarkSession| {
        let mut values: Vec<_> = session
            .run_ids
            .iter()
            .filter_map(|&r| {
                let m = get_run_metadata(session, r);
                (m.group == "dot_product" && m.variant == "Original").then_some((
                    m.scenario_id,
                    m.context,
                    m.seed,
                ))
            })
            .collect();
        values.sort();
        values
    };
    assert_eq!(collect(&session), collect(&filtered));
    let changed = init_session(&Config {
        seed: Some(43),
        ..config
    });
    assert_ne!(collect(&session), collect(&changed));
}

#[test]
fn validation_ignores_benchmark_repetition_count_and_uses_unselected_reference() {
    let config = Config {
        filter_scenario: Some(vec!["dot_product".into()]),
        filter_variant: Some(vec!["ScalarOpt".into()]),
        seed: Some(42),
        sample: Some(1),
        ..Default::default()
    };
    let once = verify(&config);
    let many = verify(&Config {
        sample: Some(1000),
        ..config
    });
    assert!(once.passed() && many.passed());
    assert_eq!(once.records.len(), many.records.len());
    for (a, b) in once.records.iter().zip(&many.records) {
        assert!(a.variant.contains("ScalarOpt"));
        assert_eq!((a.seed, &a.result.measured), (b.seed, &b.result.measured));
    }
}

#[test]
fn cli_separates_validation_and_timing_and_rejects_empty_selection() {
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_micro-algo"))
            .args(args)
            .output()
            .unwrap()
    };
    let validation = run(&[
        "--verify",
        "--scenario",
        "xoroshiro",
        "--variant",
        "Original",
        "--seed",
        "42",
    ]);
    assert!(validation.status.success());
    let report = String::from_utf8(validation.stdout).unwrap();
    assert!(report.contains("byte-entropy") && report.contains("0 failures"));
    assert!(!report.contains("Average"));
    let timing = run(&[
        "--scenario",
        "xoroshiro",
        "--variant",
        "Original",
        "--runs",
        "2",
        "--seed",
        "42",
    ]);
    assert!(timing.status.success());
    let report = String::from_utf8(timing.stdout).unwrap();
    assert!(report.contains("Average") && report.contains("Seed: 42"));
    assert!(
        !report.contains("PASS") && !report.contains("Verified") && !report.contains("entropy")
    );
    assert!(!run(&["--verify", "--scenario", "missing-family"])
        .status
        .success());
    assert!(!run(&["--seed", "invalid"]).status.success());
    assert!(!run(&["--runs", "0"]).status.success());
}
