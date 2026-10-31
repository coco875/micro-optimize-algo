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
    assert!(!report.contains("Warming up") && !report.contains("CPU pinning"));
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
    let variants = init_session(&Config {
        sample: Some(1),
        filter_scenario: Some(vec!["xoroshiro".into()]),
        filter_variant: Some(vec!["Original".into()]),
        ..Default::default()
    })
    .run_ids
    .len();
    assert!(report.contains(&format!("Warming up {} tasks", 10 * variants)));
    assert!(report.contains("(2 runs)"));
    assert!(
        !report.contains("PASS") && !report.contains("Verified") && !report.contains("entropy")
    );
    assert!(!run(&["--verify", "--scenario", "missing-family"])
        .status
        .success());
    assert!(!run(&["--seed", "invalid"]).status.success());
    assert!(!run(&["--runs", "0"]).status.success());
    for args in [
        vec!["--warmup"],
        vec!["--warmup", "-1"],
        vec!["--warmup", "invalid"],
        vec!["--pin"],
        vec!["--pin", "invalid"],
    ] {
        assert!(!run(&args).status.success());
    }
}

#[test]
fn scalar_math_families_have_std_references_and_complete_accuracy_reports() {
    let config = Config {
        seed: Some(42),
        filter_tag: Some(vec!["scalar-math".into()]),
        ..Default::default()
    };
    let report = verify(&config);
    assert!(report.passed());
    assert_eq!(report.records.len(), 72);
    for group in ["cos", "sin", "tan", "pow2", "pow", "log"] {
        assert_eq!(
            micro_optimize_algo::registry::reference_variant(group),
            Some("Std")
        );
        let records: Vec<_> = report.records.iter().filter(|r| r.group == group).collect();
        assert_eq!(
            records.len(),
            12,
            "{group}: two domains, six implementations"
        );
        for record in records {
            let metrics = &record.result.metrics;
            assert_eq!(metrics["samples"], "65536");
            assert_eq!(metrics["nonfinite"], "0");
            assert!(metrics["max_abs"].parse::<f64>().unwrap().is_finite());
            assert!(metrics["max_rel"].parse::<f64>().unwrap().is_finite());
            if record.variant == "Std" {
                assert_eq!(metrics["mismatches"], "0");
                assert_eq!(metrics["max_abs"], "0");
                assert_eq!(metrics["max_rel"], "0");
            }
        }
    }
}

#[test]
fn log2_timing_uses_std_as_baseline_when_selected() {
    for (variants, baseline) in [("BitApprox,Std", "Std"), ("BitApprox", "BitApprox")] {
        let output = Command::new(env!("CARGO_BIN_EXE_micro-algo"))
            .args([
                "--scenario",
                "log2",
                "--variant",
                variants,
                "--runs",
                "1",
                "--warmup",
                "0",
                "--seed",
                "42",
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let report = String::from_utf8(output.stdout).unwrap();
        let tables: Vec<_> = report.split("Context / Size:").skip(1).collect();
        assert!(!tables.is_empty());
        for table in tables {
            assert!(table.contains(&format!("Speedup baseline: {baseline}")));
            let first_variant = table
                .lines()
                .find(|line| matches!(line.split_whitespace().next(), Some("Std" | "BitApprox")))
                .unwrap();
            assert_eq!(first_variant.split_whitespace().next(), Some(baseline));
            assert!(first_variant.contains("1.00x"));
        }
    }
}

#[test]
fn cli_exports_validation_metrics_and_reports_export_errors() {
    let dir =
        std::env::temp_dir().join(format!("micro-algo-validation-csv-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_micro-algo"))
            .current_dir(&dir)
            .args([
                "--verify",
                "--scenario",
                "log2",
                "--seed",
                "42",
                "--variant",
                "BitApprox,Std",
            ])
            .args(args)
            .output()
            .unwrap()
    };
    let output = run(&["--csv", "accuracy.csv"]);
    assert!(output.status.success(), "{output:?}");
    let csv = std::fs::read_to_string(dir.join("accuracy.csv")).unwrap();
    let mut lines = csv.lines();
    assert_eq!(lines.next().unwrap(), "group,scenario,variant,case,fixture_seed,check,passed,measured,threshold,reason,metric_max_abs,metric_mismatches,metric_rmse,metric_samples");
    let rows: Vec<_> = lines.collect();
    assert_eq!(rows.len(), 4);
    for row in rows {
        let metrics: Vec<_> = row.rsplitn(5, ',').collect();
        assert_eq!(metrics[0], "65536");
        let rmse = metrics[1].parse::<f64>().unwrap();
        let max_abs = metrics[3].parse::<f64>().unwrap();
        if row.starts_with("log2,Log2,Std,") {
            assert_eq!(metrics[2], "0");
            assert_eq!(rmse, 0.0);
            assert_eq!(max_abs, 0.0);
        } else {
            assert!(row.starts_with("log2,Log2,BitApprox,"));
            assert_eq!(metrics[2], "65536");
            assert!(rmse > 0.6);
            assert!(max_abs > 0.99);
        }
    }
    assert!(!dir.join("accuracy.samples.csv").exists());

    let filtered = run(&["--csv", "filtered.csv", "--variant", "BitApprox"]);
    assert!(filtered.status.success(), "{filtered:?}");
    let csv = std::fs::read_to_string(dir.join("filtered.csv")).unwrap();
    assert_eq!(csv.lines().count(), 3);
    assert!(csv
        .lines()
        .skip(1)
        .all(|row| row.starts_with("log2,Log2,BitApprox,")));

    let error = run(&["--csv", "missing-directory/accuracy.csv"]);
    assert!(!error.status.success());
    assert!(String::from_utf8(error.stderr)
        .unwrap()
        .contains("Failed to export validation CSV"));

    let empty = run(&["--csv", "empty.csv", "--variant", "missing-variant"]);
    assert!(!empty.status.success());
    let csv = std::fs::read_to_string(dir.join("empty.csv")).unwrap();
    assert_eq!(csv.lines().count(), 1);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_honors_warmup_and_pinning_overrides_without_exporting_warmups() {
    let dir = std::env::temp_dir().join(format!("micro-algo-warmup-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let config = "sample = 2\nseed = 42\nwarmup = 1\npin = 'global'\nfilter_scenario = ['xoroshiro']\nfilter_variant = ['Original']\n";
    std::fs::write(dir.join("config.toml"), config).unwrap();
    let session = init_session(&toml::from_str(config).unwrap());
    let variants = session.run_ids.len() / 2;
    let run = |extra: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_micro-algo"))
            .current_dir(&dir)
            .args(["--csv", "results.csv"])
            .args(extra)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let report = String::from_utf8(output.stdout).unwrap();
        let csv = std::fs::read_to_string(dir.join("results.csv")).unwrap();
        assert_eq!(csv.lines().count(), variants + 1);
        let samples = std::fs::read_to_string(dir.join("results.samples.csv")).unwrap();
        assert_eq!(samples.lines().count(), session.run_ids.len() + 1);
        for (index, line) in samples.lines().skip(1).enumerate() {
            let fields: Vec<_> = line.split(',').collect();
            let meta = get_run_metadata(&session, session.run_ids[index]);
            assert_eq!(fields[2], meta.variant);
            assert_eq!(fields[5], "42");
            assert_eq!(fields[6].parse::<u64>().unwrap(), meta.seed);
            assert_eq!(fields[8].parse::<usize>().unwrap(), index);
            assert_eq!(fields[9], "true");
        }
        assert!(report.contains(&format!("Running {} tasks", session.run_ids.len())));
        assert!(report.contains("(2 runs)"));
        report
    };
    assert!(run(&[]).contains(&format!("Warming up {variants} tasks")));
    assert!(run(&["--warmup", "3", "--pin", "per-call"])
        .contains(&format!("Warming up {} tasks", 3 * variants)));
    assert!(run(&["--warmup", "0", "--pin", "per-execution"]).contains("Warming up 0 tasks"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn exported_statistics_can_be_recomputed_from_all_raw_samples() {
    let dir = std::env::temp_dir().join(format!("micro-algo-statistics-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    for runs in [1, 12] {
        let output = Command::new(env!("CARGO_BIN_EXE_micro-algo"))
            .current_dir(&dir)
            .args([
                "--scenario",
                "xoroshiro",
                "--variant",
                "Original",
                "--seed",
                "42",
                "--warmup",
                "0",
                "--runs",
                &runs.to_string(),
                "--filter",
                "--csv",
                "results.csv",
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let report = std::fs::read_to_string(dir.join("results.csv")).unwrap();
        let samples = std::fs::read_to_string(dir.join("results.samples.csv")).unwrap();
        let raw: Vec<Vec<_>> = samples
            .lines()
            .skip(1)
            .map(|line| line.split(',').collect())
            .collect();
        for line in report.lines().skip(1) {
            let fields: Vec<_> = line.split(',').collect();
            assert_eq!(fields.len(), 17);
            let all: Vec<_> = raw.iter().filter(|row| row[2] == fields[2]).collect();
            assert_eq!(all.len(), runs);
            let retained: Vec<_> = all
                .iter()
                .filter(|row| row[9] == "true")
                .map(|row| row[4].parse::<u64>().unwrap())
                .collect();
            assert_eq!(retained.len(), if runs == 12 { 10 } else { 1 });
            let stats =
                micro_optimize_algo::utils::statistics::TimingStatistics::new(&retained).unwrap();
            assert_eq!(fields[4].parse::<f64>().unwrap(), stats.mean);
            assert_eq!(fields[7].parse::<usize>().unwrap(), stats.samples);
            assert_eq!(fields[8].parse::<usize>().unwrap(), runs);
            assert_eq!(fields[9].parse::<f64>().unwrap(), stats.median);
            assert_eq!(fields[10].parse::<f64>().unwrap(), stats.p95);
            assert_eq!(fields[11].parse::<u64>().unwrap(), stats.min);
            assert_eq!(fields[12].parse::<u64>().unwrap(), stats.max);
            if runs == 1 {
                assert!(fields[13..].iter().all(|field| field.is_empty()));
                assert!(String::from_utf8_lossy(&output.stdout).contains("N/A"));
            } else {
                assert_eq!(fields[13].parse::<f64>().unwrap(), stats.stddev.unwrap());
                let (low, high) = stats.mean_ci95.unwrap();
                assert_eq!(fields[15].parse::<f64>().unwrap(), low);
                assert_eq!(fields[16].parse::<f64>().unwrap(), high);
            }
        }
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn benchmark_restores_original_cpu_affinity_for_both_pinning_strategies() {
    use micro_optimize_algo::utils::{
        cpu_affinity::CpuPinGuard, runner::run_benchmarks, timer::PinStrategy,
    };
    let affinity = || unsafe {
        let mut set: libc::cpu_set_t = std::mem::zeroed();
        assert_eq!(
            libc::sched_getaffinity(0, std::mem::size_of_val(&set), &mut set),
            0
        );
        (0..libc::CPU_SETSIZE as usize)
            .filter(|&core| libc::CPU_ISSET(core, &set))
            .collect::<Vec<_>>()
    };
    let original = affinity();
    {
        let guard = CpuPinGuard::new();
        if let Some(core) = guard.core_id() {
            assert_eq!(affinity(), vec![core]);
        }
    }
    assert_eq!(affinity(), original);
    for pin in [PinStrategy::Global, PinStrategy::PerExecution] {
        run_benchmarks(
            toml::from_str(&format!(
                "sample = 1\nseed = 42\nwarmup = 1\npin = '{}'\nfilter_scenario = ['xoroshiro']\nfilter_variant = ['Original']\n[xoroshiro]\nsizes = [8]",
                if pin == PinStrategy::Global { "global" } else { "per-call" }
            ))
            .unwrap(),
            None,
            false,
        );
        assert_eq!(affinity(), original);
    }
}
