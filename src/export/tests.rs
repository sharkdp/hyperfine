use super::Exporter;
use crate::benchmark::benchmark_result::{BenchmarkResult, BenchmarkRun};
use crate::export::asciidoc::AsciidocExporter;
use crate::export::orgmode::OrgmodeExporter;
use crate::util::units::Unit;
use crate::{export::markdown::MarkdownExporter, options::SortOrder};
use std::collections::BTreeMap;

fn get_output<E: Exporter + Default>(
    results: &[BenchmarkResult],
    unit: Option<Unit>,
    sort_order: SortOrder,
) -> String {
    let exporter = E::default();
    String::from_utf8(exporter.serialize(results, unit, sort_order).unwrap()).unwrap()
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
            runs: vec![
                BenchmarkRun {
                    wall_clock_time: 0.09,
                    user_time: 0.09,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 0.10,
                    user_time: 0.10,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 0.14,
                    user_time: 0.14,
                    system_time: 0.,
                },
            ],
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 2"),
            command_with_unused_parameters: String::from("sleep 2"),
            runs: vec![
                BenchmarkRun {
                    wall_clock_time: 2.0,
                    user_time: 2.0,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 3.0,
                    user_time: 3.0,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 4.0,
                    user_time: 4.0,
                    system_time: 0.,
                },
            ],
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
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
            runs: vec![
                BenchmarkRun {
                    wall_clock_time: 2.1,
                    user_time: 2.1,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 2.2,
                    user_time: 2.2,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 2.3,
                    user_time: 2.3,
                    system_time: 0.,
                },
            ],
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            command_with_unused_parameters: String::from("sleep 0.1"),
            runs: vec![
                BenchmarkRun {
                    wall_clock_time: 0.1,
                    user_time: 0.1,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 0.2,
                    user_time: 0.2,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 0.3,
                    user_time: 0.3,
                    system_time: 0.,
                },
            ],
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
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
            runs: vec![
                BenchmarkRun {
                    wall_clock_time: 2.1,
                    user_time: 2.1,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 2.2,
                    user_time: 2.2,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 2.3,
                    user_time: 2.3,
                    system_time: 0.,
                },
            ],
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            command_with_unused_parameters: String::from("sleep 0.1"),
            runs: vec![
                BenchmarkRun {
                    wall_clock_time: 0.1,
                    user_time: 0.1,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 0.2,
                    user_time: 0.2,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 0.3,
                    user_time: 0.3,
                    system_time: 0.,
                },
            ],
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
    ];

    insta::assert_snapshot!(get_output::<MarkdownExporter>(&timing_results, Some(Unit::MilliSecond), SortOrder::Command), @r#"
    | Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
    |:---|---:|---:|---:|---:|
    | `sleep 2` | 2200.0 ± 100.0 | 2100.0 | 2300.0 | 11.00 ± 5.52 |
    | `sleep 0.1` | 200.0 ± 100.0 | 100.0 | 300.0 | 1.00 |
    "#);

    insta::assert_snapshot!(get_output::<AsciidocExporter>(&timing_results, Some(Unit::MilliSecond), SortOrder::Command), @r#"
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

    insta::assert_snapshot!(get_output::<OrgmodeExporter>(&timing_results, Some(Unit::MilliSecond), SortOrder::Command), @r#"
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
            runs: vec![
                BenchmarkRun {
                    wall_clock_time: 2.01,
                    user_time: 2.01,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 2.02,
                    user_time: 2.02,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 2.03,
                    user_time: 2.03,
                    system_time: 0.,
                },
            ],
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            command_with_unused_parameters: String::from("sleep 0.1"),
            runs: vec![
                BenchmarkRun {
                    wall_clock_time: 0.11,
                    user_time: 0.11,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 0.12,
                    user_time: 0.12,
                    system_time: 0.,
                },
                BenchmarkRun {
                    wall_clock_time: 0.13,
                    user_time: 0.13,
                    system_time: 0.,
                },
            ],
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
    ];

    insta::assert_snapshot!(get_output::<MarkdownExporter>(&results, Some(Unit::Second), SortOrder::Command), @r#"
    | Command | Mean [s] | Min [s] | Max [s] | Relative |
    |:---|---:|---:|---:|---:|
    | `sleep 2` | 2.020 ± 0.010 | 2.010 | 2.030 | 16.83 ± 1.41 |
    | `sleep 0.1` | 0.120 ± 0.010 | 0.110 | 0.130 | 1.00 |
    "#);

    insta::assert_snapshot!(get_output::<MarkdownExporter>(&results, Some(Unit::Second), SortOrder::MeanTime), @r#"
    | Command | Mean [s] | Min [s] | Max [s] | Relative |
    |:---|---:|---:|---:|---:|
    | `sleep 0.1` | 0.120 ± 0.010 | 0.110 | 0.130 | 1.00 |
    | `sleep 2` | 2.020 ± 0.010 | 2.010 | 2.030 | 16.83 ± 1.41 |
    "#);

    insta::assert_snapshot!(get_output::<AsciidocExporter>(&results, Some(Unit::Second), SortOrder::Command), @r#"
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
