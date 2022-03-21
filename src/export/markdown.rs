use super::markup::Alignment;
use crate::export::{markup::MarkupExporter, BenchmarkResult, Exporter};
use crate::util::units::Unit;
use anyhow::Result;

#[derive(Default)]
pub struct MarkdownExporter {}

impl MarkupExporter for MarkdownExporter {
    fn table_row(&self, cells: &[&str]) -> String {
        format!("| {} |\n", cells.join(" | "))
    }

    fn table_divider(&self, cell_aligmnents: &[Alignment]) -> String {
        format!(
            "|{}\n",
            cell_aligmnents
                .iter()
                .map(|a| match a {
                    Alignment::Left => ":---|",
                    Alignment::Right => "---:|",
                })
                .collect::<String>()
        )
    }

    fn command(&self, cmd: &str) -> String {
        format!("`{}`", cmd)
    }
}

#[derive(Default)]
pub struct MarkdownRunsExporter(MarkdownExporter);

impl Exporter for MarkdownRunsExporter {
    fn serialize(&self, results: &[BenchmarkResult], _unit: Option<Unit>) -> Result<Vec<u8>> {
        let unit = Unit::Second;
        let table = self.0.table_runs(results, unit);
        Ok(table.as_bytes().to_vec())
    }
}

/// Check Markdown-based data row formatting
#[test]
fn test_markdown_formatter_table_data() {
    let formatter = MarkdownExporter::default();

    assert_eq!(formatter.table_row(&["a", "b", "c"]), "| a | b | c |\n");
}

/// Check Markdown-based horizontal line formatting
#[test]
fn test_markdown_formatter_table_divider() {
    let formatter = MarkdownExporter::default();

    let divider = formatter.table_divider(&[Alignment::Left, Alignment::Right, Alignment::Left]);
    assert_eq!(divider, "|:---|---:|:---|\n");
}

/// Test helper function to create unit-based header and horizontal line
/// independently from the markup functionality for Markdown.
#[cfg(test)]
fn cfg_test_table_header(unit_short_name: String) -> String {
    format!(
        "| Command | Mean [{unit}] | Min [{unit}] | Max [{unit}] | Relative |\n|:---|---:|---:|---:|---:|\n",
        unit = unit_short_name
    )
}

/// Ensure the markdown output includes the table header and the multiple
/// benchmark results as a table. The list of actual times is not included
/// in the output.
///
/// This also demonstrates that the first entry's units (ms) are used to set
/// the units for all entries when the time unit is not given.
#[test]
fn test_markdown_format_ms() {
    use super::Exporter;
    use crate::benchmark::benchmark_result::BenchmarkResult;
    use std::collections::BTreeMap;
    let exporter = MarkdownExporter::default();

    let timing_results = vec![
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            mean: 0.1057,
            stddev: Some(0.0016),
            median: 0.1057,
            user: 0.0009,
            system: 0.0011,
            min: 0.1023,
            max: 0.1080,
            times: Some(vec![0.1, 0.1, 0.1]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 2"),
            mean: 2.0050,
            stddev: Some(0.0020),
            median: 2.0050,
            user: 0.0009,
            system: 0.0012,
            min: 2.0020,
            max: 2.0080,
            times: Some(vec![2.0, 2.0, 2.0]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
    ];

    let actual = String::from_utf8(exporter.serialize(&timing_results, None).unwrap()).unwrap();
    let expect = format!(
        "{}\
| `sleep 0.1` | 105.7 ± 1.6 | 102.3 | 108.0 | 1.00 |
| `sleep 2` | 2005.0 ± 2.0 | 2002.0 | 2008.0 | 18.97 ± 0.29 |
",
        cfg_test_table_header("ms".to_string())
    );

    assert_eq!(expect, actual);
}

/// This (again) demonstrates that the first entry's units (s) are used to set
/// the units for all entries when the time unit is not given.
#[test]
fn test_markdown_format_s() {
    use super::Exporter;
    use crate::benchmark::benchmark_result::BenchmarkResult;
    use std::collections::BTreeMap;
    let exporter = MarkdownExporter::default();

    let timing_results = vec![
        BenchmarkResult {
            command: String::from("sleep 2"),
            mean: 2.0050,
            stddev: Some(0.0020),
            median: 2.0050,
            user: 0.0009,
            system: 0.0012,
            min: 2.0020,
            max: 2.0080,
            times: Some(vec![2.0, 2.0, 2.0]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            mean: 0.1057,
            stddev: Some(0.0016),
            median: 0.1057,
            user: 0.0009,
            system: 0.0011,
            min: 0.1023,
            max: 0.1080,
            times: Some(vec![0.1, 0.1, 0.1]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
    ];

    let actual = String::from_utf8(exporter.serialize(&timing_results, None).unwrap()).unwrap();
    let expect = format!(
        "{}\
| `sleep 2` | 2.005 ± 0.002 | 2.002 | 2.008 | 18.97 ± 0.29 |
| `sleep 0.1` | 0.106 ± 0.002 | 0.102 | 0.108 | 1.00 |
",
        cfg_test_table_header("s".to_string())
    );

    assert_eq!(expect, actual);
}

/// The given time unit (s) is used to set the units for all entries.
#[test]
fn test_markdown_format_time_unit_s() {
    use super::Exporter;
    use crate::benchmark::benchmark_result::BenchmarkResult;
    use crate::util::units::Unit;
    use std::collections::BTreeMap;
    let exporter = MarkdownExporter::default();

    let timing_results = vec![
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            mean: 0.1057,
            stddev: Some(0.0016),
            median: 0.1057,
            user: 0.0009,
            system: 0.0011,
            min: 0.1023,
            max: 0.1080,
            times: Some(vec![0.1, 0.1, 0.1]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 2"),
            mean: 2.0050,
            stddev: Some(0.0020),
            median: 2.0050,
            user: 0.0009,
            system: 0.0012,
            min: 2.0020,
            max: 2.0080,
            times: Some(vec![2.0, 2.0, 2.0]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
    ];

    let actual = String::from_utf8(
        exporter
            .serialize(&timing_results, Some(Unit::Second))
            .unwrap(),
    )
    .unwrap();
    let expect = format!(
        "{}\
| `sleep 0.1` | 0.106 ± 0.002 | 0.102 | 0.108 | 1.00 |
| `sleep 2` | 2.005 ± 0.002 | 2.002 | 2.008 | 18.97 ± 0.29 |
",
        cfg_test_table_header("s".to_string())
    );

    assert_eq!(expect, actual);
}

/// This (again) demonstrates that the given time unit (ms) is used to set
/// the units for all entries.
#[test]
fn test_markdown_format_time_unit_ms() {
    use super::Exporter;
    use crate::benchmark::benchmark_result::BenchmarkResult;
    use crate::util::units::Unit;
    use std::collections::BTreeMap;
    let exporter = MarkdownExporter::default();

    let timing_results = vec![
        BenchmarkResult {
            command: String::from("sleep 2"),
            mean: 2.0050,
            stddev: Some(0.0020),
            median: 2.0050,
            user: 0.0009,
            system: 0.0012,
            min: 2.0020,
            max: 2.0080,
            times: Some(vec![2.0, 2.0, 2.0]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            mean: 0.1057,
            stddev: Some(0.0016),
            median: 0.1057,
            user: 0.0009,
            system: 0.0011,
            min: 0.1023,
            max: 0.1080,
            times: Some(vec![0.1, 0.1, 0.1]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
    ];

    let actual = String::from_utf8(
        exporter
            .serialize(&timing_results, Some(Unit::MilliSecond))
            .unwrap(),
    )
    .unwrap();
    let expect = format!(
        "{}\
| `sleep 2` | 2005.0 ± 2.0 | 2002.0 | 2008.0 | 18.97 ± 0.29 |
| `sleep 0.1` | 105.7 ± 1.6 | 102.3 | 108.0 | 1.00 |
",
        cfg_test_table_header("ms".to_string())
    );

    assert_eq!(expect, actual);
}

/// This test demonstrates the exporting of all individual timings of all runs
/// without any parameters.
#[test]
fn test_markdown_format_runs() {
    use std::collections::BTreeMap;
    let exporter = MarkdownRunsExporter::default();

    let results = vec![
        BenchmarkResult {
            command: String::from("sleep 2"),
            mean: 2.0050,
            stddev: Some(0.0020),
            median: 2.0050,
            user: 0.0009,
            system: 0.0012,
            min: 2.0020,
            max: 2.0080,
            times: Some(vec![2.0, 2.0, 2.0]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            mean: 0.1057,
            stddev: Some(0.0016),
            median: 0.1057,
            user: 0.0009,
            system: 0.0011,
            min: 0.1023,
            max: 0.1080,
            times: Some(vec![0.1, 0.1, 0.1]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::new(),
        },
    ];

    let formatted =
        String::from_utf8(exporter.serialize(&results, Some(Unit::Second)).unwrap()).unwrap();

    let formatted_expected = "| Command | Sample | Time [s] | Run | Parameters |
|:---|---:|---:|---:|---:|
| `sleep 2` | 1 | 2.000000000 | ok | - |
| `sleep 2` | 2 | 2.000000000 | ok | - |
| `sleep 2` | 3 | 2.000000000 | ok | - |
| `sleep 0.1` | 4 | 0.100000000 | ok | - |
| `sleep 0.1` | 5 | 0.100000000 | ok | - |
| `sleep 0.1` | 6 | 0.100000000 | ok | - |
";

    assert_eq!(formatted_expected, formatted);
}

/// This test demonstrates the exporting of all individual timings of all runs
/// with some parameters.
#[test]
fn test_markdown_format_runs_parameters() {
    use std::collections::BTreeMap;
    let exporter = MarkdownRunsExporter::default();

    let results = vec![
        BenchmarkResult {
            command: String::from("sleep 0.1"),
            mean: 0.1057,
            stddev: Some(0.0016),
            median: 0.1057,
            user: 0.0009,
            system: 0.0011,
            min: 0.1023,
            max: 0.1080,
            times: Some(vec![0.1, 0.1, 0.1]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::from([("time".to_string(), "0.1".to_string())]),
        },
        BenchmarkResult {
            command: String::from("sleep 2"),
            mean: 2.0050,
            stddev: Some(0.0020),
            median: 2.0050,
            user: 0.0009,
            system: 0.0012,
            min: 2.0020,
            max: 2.0080,
            times: Some(vec![2.0, 2.0, 2.0]),
            exit_codes: vec![Some(0), Some(0), Some(0)],
            parameters: BTreeMap::from([("time".to_string(), "2".to_string())]),
        },
    ];

    let formatted =
        String::from_utf8(exporter.serialize(&results, Some(Unit::Second)).unwrap()).unwrap();

    let formatted_expected = "| Command | Sample | Time [s] | Run | Parameters |
|:---|---:|---:|---:|---:|
| `sleep 0.1` | 1 | 0.100000000 | ok | time=0.1 |
| `sleep 0.1` | 2 | 0.100000000 | ok | time=0.1 |
| `sleep 0.1` | 3 | 0.100000000 | ok | time=0.1 |
| `sleep 2` | 4 | 2.000000000 | ok | time=2 |
| `sleep 2` | 5 | 2.000000000 | ok | time=2 |
| `sleep 2` | 6 | 2.000000000 | ok | time=2 |
";

    assert_eq!(formatted_expected, formatted);
}
