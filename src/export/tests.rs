use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::util::units::Unit;
use crate::{export::markdown::MarkdownExporter, options::SortOrder};
use std::collections::BTreeMap;

fn get_markdown_output(
    results: &[BenchmarkResult],
    unit: Option<Unit>,
    sort_order: SortOrder,
) -> String {
    let exporter = MarkdownExporter::default();
    String::from_utf8(exporter.serialize(results, unit, sort_order).unwrap()).unwrap()
}

/// Ensure the markdown output includes the table header and the multiple
/// benchmark results as a table. The list of actual times is not included
/// in the output.
///
/// This also demonstrates that the first entry's units (ms) are used to set
/// the units for all entries when the time unit is not given.
#[test]
fn test_markdown_format_ms() {
    let results = [
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            command_with_unused_parameters: String::from("sleep 0.1"),
            mean: 0.1057,
            stddev: Some(0.0016),
            median: 0.1057,
            user: 0.0009,
            system: 0.0011,
            min: 0.1023,
            max: 0.1080,
            times: Some(vec![0.1, 0.1, 0.1]),
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 2"),
            command_with_unused_parameters: String::from("sleep 2"),
            mean: 2.0050,
            stddev: Some(0.0020),
            median: 2.0050,
            user: 0.0009,
            system: 0.0012,
            min: 2.0020,
            max: 2.0080,
            times: Some(vec![2.0, 2.0, 2.0]),
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
    ];

    insta::assert_snapshot!(get_markdown_output(&results, None, SortOrder::Command), @r#"
    | Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
    |:---|---:|---:|---:|---:|
    | `sleep 0.1` | 105.7 ± 1.6 | 102.3 | 108.0 | 1.00 |
    | `sleep 2` | 2005.0 ± 2.0 | 2002.0 | 2008.0 | 18.97 ± 0.29 |
    "#);
}

/// This (again) demonstrates that the first entry's units (s) are used to set
/// the units for all entries when the time unit is not given.
#[test]
fn test_markdown_format_s() {
    let results = [
        BenchmarkResult {
            command: String::from("sleep 2"),
            command_with_unused_parameters: String::from("sleep 2"),
            mean: 2.0050,
            stddev: Some(0.0020),
            median: 2.0050,
            user: 0.0009,
            system: 0.0012,
            min: 2.0020,
            max: 2.0080,
            times: Some(vec![2.0, 2.0, 2.0]),
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            command_with_unused_parameters: String::from("sleep 0.1"),
            mean: 0.1057,
            stddev: Some(0.0016),
            median: 0.1057,
            user: 0.0009,
            system: 0.0011,
            min: 0.1023,
            max: 0.1080,
            times: Some(vec![0.1, 0.1, 0.1]),
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
    ];

    insta::assert_snapshot!(get_markdown_output(&results, None, SortOrder::Command), @r#"
    | Command | Mean [s] | Min [s] | Max [s] | Relative |
    |:---|---:|---:|---:|---:|
    | `sleep 2` | 2.005 ± 0.002 | 2.002 | 2.008 | 18.97 ± 0.29 |
    | `sleep 0.1` | 0.106 ± 0.002 | 0.102 | 0.108 | 1.00 |
    "#);
}

/// The given time unit (s) is used to set the units for all entries.
#[test]
fn test_markdown_format_time_unit_s() {
    let results = [
        BenchmarkResult {
            command: String::from("sleep 2"),
            command_with_unused_parameters: String::from("sleep 2"),
            mean: 2.0050,
            stddev: Some(0.0020),
            median: 2.0050,
            user: 0.0009,
            system: 0.0012,
            min: 2.0020,
            max: 2.0080,
            times: Some(vec![2.0, 2.0, 2.0]),
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            command_with_unused_parameters: String::from("sleep 0.1"),
            mean: 0.1057,
            stddev: Some(0.0016),
            median: 0.1057,
            user: 0.0009,
            system: 0.0011,
            min: 0.1023,
            max: 0.1080,
            times: Some(vec![0.1, 0.1, 0.1]),
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
    ];

    insta::assert_snapshot!(get_markdown_output(&results, Some(Unit::Second), SortOrder::Command), @r#"
        | Command | Mean [s] | Min [s] | Max [s] | Relative |
        |:---|---:|---:|---:|---:|
        | `sleep 2` | 2.005 ± 0.002 | 2.002 | 2.008 | 18.97 ± 0.29 |
        | `sleep 0.1` | 0.106 ± 0.002 | 0.102 | 0.108 | 1.00 |
        "#);

    insta::assert_snapshot!(get_markdown_output(&results, Some(Unit::Second), SortOrder::MeanTime), @r#"
        | Command | Mean [s] | Min [s] | Max [s] | Relative |
        |:---|---:|---:|---:|---:|
        | `sleep 0.1` | 0.106 ± 0.002 | 0.102 | 0.108 | 1.00 |
        | `sleep 2` | 2.005 ± 0.002 | 2.002 | 2.008 | 18.97 ± 0.29 |
        "#);
}

/// This (again) demonstrates that the given time unit (ms) is used to set
/// the units for all entries.
#[test]
fn test_markdown_format_time_unit_ms() {
    let timing_results = [
        BenchmarkResult {
            command: String::from("sleep 2"),
            command_with_unused_parameters: String::from("sleep 2"),
            mean: 2.0050,
            stddev: Some(0.0020),
            median: 2.0050,
            user: 0.0009,
            system: 0.0012,
            min: 2.0020,
            max: 2.0080,
            times: Some(vec![2.0, 2.0, 2.0]),
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            command_with_unused_parameters: String::from("sleep 0.1"),
            mean: 0.1057,
            stddev: Some(0.0016),
            median: 0.1057,
            user: 0.0009,
            system: 0.0011,
            min: 0.1023,
            max: 0.1080,
            times: Some(vec![0.1, 0.1, 0.1]),
            memory_usage_byte: None,
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
    ];

    insta::assert_snapshot!(get_markdown_output(&timing_results, Some(Unit::MilliSecond), SortOrder::Command), @r#"
    | Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
    |:---|---:|---:|---:|---:|
    | `sleep 2` | 2005.0 ± 2.0 | 2002.0 | 2008.0 | 18.97 ± 0.29 |
    | `sleep 0.1` | 105.7 ± 1.6 | 102.3 | 108.0 | 1.00 |
    "#);
}
