use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::benchmark::measurement::{Measurement, Measurements};
use crate::export::asciidoc::AsciidocExporter;
use crate::export::orgmode::OrgmodeExporter;
use crate::quantity::{byte, second, Information, Time, TimeUnit, Zero};
use crate::{export::markdown::MarkdownExporter, options::SortOrder};
use std::collections::BTreeMap;
use std::process::ExitStatus;

fn get_output<E: Exporter + Default>(
    results: &[BenchmarkResult],
    unit: Option<TimeUnit>,
    sort_order: SortOrder,
) -> String {
    get_output_with_reference::<E>(results, unit, sort_order, None)
}

fn get_output_with_reference<E: Exporter + Default>(
    results: &[BenchmarkResult],
    unit: Option<TimeUnit>,
    sort_order: SortOrder,
    reference_index: Option<usize>,
) -> String {
    let exporter = E::default();
    String::from_utf8(
        exporter
            .serialize(results, unit, sort_order, reference_index)
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn test_markup_exports_identify_explicit_reference_and_direction() {
    let benchmark = |name: &str, seconds| {
        let measurement = Measurement {
            time_wall_clock: Time::new::<second>(seconds),
            ..Measurement::default()
        };
        BenchmarkResult {
            command: name.to_owned(),
            command_with_unused_parameters: name.to_owned(),
            measurements: Measurements::new(vec![measurement, measurement]),
            ..BenchmarkResult::default()
        }
    };
    let results = [
        benchmark("faster", 1.0),
        benchmark("reference", 2.0),
        benchmark("equal", 2.0),
        benchmark("slower", 3.0),
    ];

    for sort_order in [SortOrder::Command, SortOrder::MeanTime] {
        let outputs = [
            get_output_with_reference::<MarkdownExporter>(
                &results,
                Some(TimeUnit::Second),
                sort_order,
                Some(1),
            ),
            get_output_with_reference::<AsciidocExporter>(
                &results,
                Some(TimeUnit::Second),
                sort_order,
                Some(1),
            ),
            get_output_with_reference::<OrgmodeExporter>(
                &results,
                Some(TimeUnit::Second),
                sort_order,
                Some(1),
            ),
        ];
        for output in outputs {
            for expected in [
                "2.00 ± 0.00 (faster)",
                "1.00 (reference)",
                "1.00 ± 0.00 (same speed)",
                "1.50 ± 0.00 (slower)",
            ] {
                assert!(output.contains(expected), "{}", output);
            }
        }
    }
}

#[test]
fn test_markup_exports_leave_pending_reference_ratios_unavailable() {
    let results = [BenchmarkResult {
        command: "sleep 1".into(),
        command_with_unused_parameters: "sleep 1".into(),
        measurements: Measurements::new(vec![Measurement {
            time_wall_clock: Time::new::<second>(1.0),
            ..Measurement::default()
        }]),
        ..BenchmarkResult::default()
    }];
    for sort_order in [SortOrder::Command, SortOrder::MeanTime] {
        let outputs = [
            get_output_with_reference::<MarkdownExporter>(
                &results,
                Some(TimeUnit::Second),
                sort_order,
                Some(1),
            ),
            get_output_with_reference::<AsciidocExporter>(
                &results,
                Some(TimeUnit::Second),
                sort_order,
                Some(1),
            ),
            get_output_with_reference::<OrgmodeExporter>(
                &results,
                Some(TimeUnit::Second),
                sort_order,
                Some(1),
            ),
        ];
        for output in outputs {
            assert!(output.contains("N/A"), "{}", output);
            assert!(!output.contains("(reference)"), "{}", output);
        }
    }
}

#[test]
fn test_markup_exports_with_zero_reference() {
    let benchmark = |name: &str, seconds| BenchmarkResult {
        command: name.to_owned(),
        command_with_unused_parameters: name.to_owned(),
        measurements: Measurements::new(vec![Measurement {
            time_wall_clock: Time::new::<second>(seconds),
            ..Measurement::default()
        }]),
        ..BenchmarkResult::default()
    };
    let results = [
        benchmark("other zero", 0.0),
        benchmark("reference", 0.0),
        benchmark("slower", 1.0),
    ];
    for sort_order in [SortOrder::Command, SortOrder::MeanTime] {
        for reference in [Some(1), Some(3), None] {
            let outputs = [
                get_output_with_reference::<MarkdownExporter>(
                    &results,
                    Some(TimeUnit::Second),
                    sort_order,
                    reference,
                ),
                get_output_with_reference::<AsciidocExporter>(
                    &results,
                    Some(TimeUnit::Second),
                    sort_order,
                    reference,
                ),
                get_output_with_reference::<OrgmodeExporter>(
                    &results,
                    Some(TimeUnit::Second),
                    sort_order,
                    reference,
                ),
            ];
            for output in outputs {
                match reference {
                    Some(1) => {
                        assert_eq!(output.matches("N/A").count(), 1, "{}", output);
                        assert!(output.contains("1.00 (reference)"), "{}", output);
                        assert!(output.contains("inf (slower)"), "{}", output);
                    }
                    Some(_) => assert_eq!(output.matches("N/A").count(), 3, "{}", output),
                    None => {
                        assert!(!output.contains("N/A"), "{}", output);
                        assert!(output.contains("inf"), "{}", output);
                        assert!(!output.contains("(reference)"), "{}", output);
                    }
                }
            }
        }
    }
}

/// Ensure the makrup output includes the table header and the multiple
/// benchmark results as a table. The list of actual times is not included
/// in the output.
///
/// This also demonstrates that the first entry's units (ms) are used to set
/// the units for all entries when the time unit is not specified.
#[test]
fn test_markup_export_auto_ms() {
    let results = [
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            command_with_unused_parameters: String::from("sleep 0.1"),
            measurements: Measurements::new(vec![
                Measurement {
                    time_wall_clock: Time::new::<second>(0.09),
                    time_user: Time::new::<second>(0.09),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(0.10),
                    time_user: Time::new::<second>(0.10),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(0.14),
                    time_user: Time::new::<second>(0.14),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
            ]),
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 2"),
            command_with_unused_parameters: String::from("sleep 2"),
            measurements: Measurements::new(vec![
                Measurement {
                    time_wall_clock: Time::new::<second>(2.0),
                    time_user: Time::new::<second>(2.0),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(3.0),
                    time_user: Time::new::<second>(3.0),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(4.0),
                    time_user: Time::new::<second>(4.0),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
            ]),
            parameters: BTreeMap::new(),
        },
    ];

    insta::assert_snapshot!(get_output::<MarkdownExporter>(&results, None, SortOrder::Command), @r#"
    | Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
    |:---|---:|---:|---:|---:|
    | `sleep 0.1` | 110.0 ± 26.5 | 90.0 | 140.0 | 1.00 |
    | `sleep 2` | 3000.0 ± 1000.0 | 2000.0 | 4000.0 | 27.27 ± 11.21 |
    "#);

    insta::assert_snapshot!(get_output::<AsciidocExporter>(&results, None, SortOrder::Command), @r#"
    [cols="<,>,>,>,>"]
    |===
    | Command 
    | Mean [ms] 
    | Min [ms] 
    | Max [ms] 
    | Relative 

    | `sleep 0.1` 
    | 110.0 ± 26.5 
    | 90.0 
    | 140.0 
    | 1.00 

    | `sleep 2` 
    | 3000.0 ± 1000.0 
    | 2000.0 
    | 4000.0 
    | 27.27 ± 11.21 
    |===
    "#);

    insta::assert_snapshot!(get_output::<OrgmodeExporter>(&results, None, SortOrder::Command), @r#"
    | Command  |  Mean [ms] |  Min [ms] |  Max [ms] |  Relative |
    |--+--+--+--+--|
    | =sleep 0.1=  |  110.0 ± 26.5 |  90.0 |  140.0 |  1.00 |
    | =sleep 2=  |  3000.0 ± 1000.0 |  2000.0 |  4000.0 |  27.27 ± 11.21 |
    "#);
}

/// This (again) demonstrates that the first entry's units (s) are used to set
/// the units for all entries when the time unit is not given.
#[test]
fn test_markup_export_auto_s() {
    let results = [
        BenchmarkResult {
            command: String::from("sleep 2"),
            command_with_unused_parameters: String::from("sleep 2"),
            measurements: Measurements::new(vec![
                Measurement {
                    time_wall_clock: Time::new::<second>(2.1),
                    time_user: Time::new::<second>(2.1),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(2.2),
                    time_user: Time::new::<second>(2.2),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(2.3),
                    time_user: Time::new::<second>(2.3),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
            ]),
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            command_with_unused_parameters: String::from("sleep 0.1"),
            measurements: Measurements::new(vec![
                Measurement {
                    time_wall_clock: Time::new::<second>(0.1),
                    time_user: Time::new::<second>(0.1),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(0.2),
                    time_user: Time::new::<second>(0.2),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(0.3),
                    time_user: Time::new::<second>(0.3),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
            ]),
            parameters: BTreeMap::new(),
        },
    ];

    insta::assert_snapshot!(get_output::<MarkdownExporter>(&results, None, SortOrder::Command), @r#"
    | Command | Mean [s] | Min [s] | Max [s] | Relative |
    |:---|---:|---:|---:|---:|
    | `sleep 2` | 2.200 ± 0.100 | 2.100 | 2.300 | 11.00 ± 5.52 |
    | `sleep 0.1` | 0.200 ± 0.100 | 0.100 | 0.300 | 1.00 |
    "#);

    insta::assert_snapshot!(get_output::<AsciidocExporter>(&results, None, SortOrder::Command), @r#"
    [cols="<,>,>,>,>"]
    |===
    | Command 
    | Mean [s] 
    | Min [s] 
    | Max [s] 
    | Relative 

    | `sleep 2` 
    | 2.200 ± 0.100 
    | 2.100 
    | 2.300 
    | 11.00 ± 5.52 

    | `sleep 0.1` 
    | 0.200 ± 0.100 
    | 0.100 
    | 0.300 
    | 1.00 
    |===
    "#);

    insta::assert_snapshot!(get_output::<OrgmodeExporter>(&results, None, SortOrder::Command), @r#"
    | Command  |  Mean [s] |  Min [s] |  Max [s] |  Relative |
    |--+--+--+--+--|
    | =sleep 2=  |  2.200 ± 0.100 |  2.100 |  2.300 |  11.00 ± 5.52 |
    | =sleep 0.1=  |  0.200 ± 0.100 |  0.100 |  0.300 |  1.00 |
    "#);
}

/// This (again) demonstrates that the given time unit (ms) is used to set
/// the units for all entries.
#[test]
fn test_markup_export_manual_ms() {
    let timing_results = [
        BenchmarkResult {
            command: String::from("sleep 2"),
            command_with_unused_parameters: String::from("sleep 2"),
            measurements: Measurements::new(vec![
                Measurement {
                    time_wall_clock: Time::new::<second>(2.1),
                    time_user: Time::new::<second>(2.1),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(2.2),
                    time_user: Time::new::<second>(2.2),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(2.3),
                    time_user: Time::new::<second>(2.3),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
            ]),
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            command_with_unused_parameters: String::from("sleep 0.1"),
            measurements: Measurements::new(vec![
                Measurement {
                    time_wall_clock: Time::new::<second>(0.1),
                    time_user: Time::new::<second>(0.1),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(0.2),
                    time_user: Time::new::<second>(0.2),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(0.3),
                    time_user: Time::new::<second>(0.3),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
            ]),
            parameters: BTreeMap::new(),
        },
    ];

    insta::assert_snapshot!(get_output::<MarkdownExporter>(&timing_results, Some(TimeUnit::MilliSecond), SortOrder::Command), @r#"
    | Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
    |:---|---:|---:|---:|---:|
    | `sleep 2` | 2200.0 ± 100.0 | 2100.0 | 2300.0 | 11.00 ± 5.52 |
    | `sleep 0.1` | 200.0 ± 100.0 | 100.0 | 300.0 | 1.00 |
    "#);

    insta::assert_snapshot!(get_output::<AsciidocExporter>(&timing_results, Some(TimeUnit::MilliSecond), SortOrder::Command), @r#"
    [cols="<,>,>,>,>"]
    |===
    | Command 
    | Mean [ms] 
    | Min [ms] 
    | Max [ms] 
    | Relative 

    | `sleep 2` 
    | 2200.0 ± 100.0 
    | 2100.0 
    | 2300.0 
    | 11.00 ± 5.52 

    | `sleep 0.1` 
    | 200.0 ± 100.0 
    | 100.0 
    | 300.0 
    | 1.00 
    |===
    "#);

    insta::assert_snapshot!(get_output::<OrgmodeExporter>(&timing_results, Some(TimeUnit::MilliSecond), SortOrder::Command), @r#"
    | Command  |  Mean [ms] |  Min [ms] |  Max [ms] |  Relative |
    |--+--+--+--+--|
    | =sleep 2=  |  2200.0 ± 100.0 |  2100.0 |  2300.0 |  11.00 ± 5.52 |
    | =sleep 0.1=  |  200.0 ± 100.0 |  100.0 |  300.0 |  1.00 |
    "#);
}

/// The given time unit (s) is used to set the units for all entries.
#[test]
fn test_markup_export_manual_s() {
    let results = [
        BenchmarkResult {
            command: String::from("sleep 2"),
            command_with_unused_parameters: String::from("sleep 2"),
            measurements: Measurements::new(vec![
                Measurement {
                    time_wall_clock: Time::new::<second>(2.01),
                    time_user: Time::new::<second>(2.01),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(2.02),
                    time_user: Time::new::<second>(2.02),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(2.03),
                    time_user: Time::new::<second>(2.03),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
            ]),
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            command_with_unused_parameters: String::from("sleep 0.1"),
            measurements: Measurements::new(vec![
                Measurement {
                    time_wall_clock: Time::new::<second>(0.11),
                    time_user: Time::new::<second>(0.11),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(0.12),
                    time_user: Time::new::<second>(0.12),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
                Measurement {
                    time_wall_clock: Time::new::<second>(0.13),
                    time_user: Time::new::<second>(0.13),
                    time_system: Time::zero(),
                    peak_memory_usage: Information::new::<byte>(1024.),
                    exit_status: ExitStatus::default(),
                },
            ]),
            parameters: BTreeMap::new(),
        },
    ];

    insta::assert_snapshot!(get_output::<MarkdownExporter>(&results, Some(TimeUnit::Second), SortOrder::Command), @r#"
    | Command | Mean [s] | Min [s] | Max [s] | Relative |
    |:---|---:|---:|---:|---:|
    | `sleep 2` | 2.020 ± 0.010 | 2.010 | 2.030 | 16.83 ± 1.41 |
    | `sleep 0.1` | 0.120 ± 0.010 | 0.110 | 0.130 | 1.00 |
    "#);

    insta::assert_snapshot!(get_output::<MarkdownExporter>(&results, Some(TimeUnit::Second), SortOrder::MeanTime), @r#"
    | Command | Mean [s] | Min [s] | Max [s] | Relative |
    |:---|---:|---:|---:|---:|
    | `sleep 0.1` | 0.120 ± 0.010 | 0.110 | 0.130 | 1.00 |
    | `sleep 2` | 2.020 ± 0.010 | 2.010 | 2.030 | 16.83 ± 1.41 |
    "#);

    insta::assert_snapshot!(get_output::<AsciidocExporter>(&results, Some(TimeUnit::Second), SortOrder::Command), @r#"
    [cols="<,>,>,>,>"]
    |===
    | Command 
    | Mean [s] 
    | Min [s] 
    | Max [s] 
    | Relative 

    | `sleep 2` 
    | 2.020 ± 0.010 
    | 2.010 
    | 2.030 
    | 16.83 ± 1.41 

    | `sleep 0.1` 
    | 0.120 ± 0.010 
    | 0.110 
    | 0.130 
    | 1.00 
    |===
    "#);
}
