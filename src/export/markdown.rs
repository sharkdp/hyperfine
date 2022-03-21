use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::benchmark::relative_speed;
use crate::export::markup::markup_results_unit;
use crate::export::markup::markup_table_data;
use crate::export::markup::markup_table_line;
use crate::export::markup::MarkupType;
use crate::output::format::format_duration_value;
use crate::util::units::Unit;

use anyhow::{anyhow, Result};

#[derive(Default)]
pub struct MarkdownExporter {}

impl Exporter for MarkdownExporter {
    fn serialize(&self, results: &[BenchmarkResult], unit: Option<Unit>) -> Result<Vec<u8>> {
        let unit = markup_results_unit(results, unit);

        let entries = relative_speed::compute(results);
        if entries.is_none() {
            return Err(anyhow!(
                "Relative speed comparison is not available for Markdown export."
            ));
        }

        // prepare table header strings
        let notation = format!("[{}]", unit.short_name());
        let mut data: Vec<Vec<_>> = vec![vec![
            format!("Command"),
            format!("Mean {}", notation),
            format!("Min {}", notation),
            format!("Max {}", notation),
            format!("Relative"),
        ]];

        for entry in entries.unwrap() {
            let measurement = &entry.result;
            // prepare data row strings
            let cmd_str = measurement.command.replace("|", "\\|");
            let mean_str = format_duration_value(measurement.mean, Some(unit)).0;
            let stddev_str = if let Some(stddev) = measurement.stddev {
                format!(" ± {}", format_duration_value(stddev, Some(unit)).0)
            } else {
                "".into()
            };
            let min_str = format_duration_value(measurement.min, Some(unit)).0;
            let max_str = format_duration_value(measurement.max, Some(unit)).0;
            let rel_str = format!("{:.2}", entry.relative_speed);
            let rel_stddev_str = if entry.is_fastest {
                "".into()
            } else if let Some(stddev) = entry.relative_speed_stddev {
                format!(" ± {:.2}", stddev)
            } else {
                "".into()
            };
            // prepare table row entries
            data.push(vec![
                format!("`{}`", cmd_str),
                format!("{}{}", mean_str, stddev_str),
                format!("{}", min_str),
                format!("{}", max_str),
                format!("{}{}", rel_str, rel_stddev_str),
            ])
        }

        let head: &Vec<String> = data.first().unwrap();
        let tail: &[Vec<String>] = &data[1..];
        let kind = MarkupType::Markdown;

        // emit header
        let mut table = markup_table_data(&kind, head);

        // emit horizontal line
        table.push_str(&markup_table_line(&kind, head.len()));

        // emit data rows
        for row in tail {
            table.push_str(&markup_table_data(&kind, row))
        }

        Ok(table.as_bytes().to_vec())
    }
}

/// Test helper function to create unit-based header and horizontal line
/// independently from the markup functionality.
#[cfg(test)]
fn test_table_header(unit_short_name: String) -> String {
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

    let formatted = String::from_utf8(exporter.serialize(&timing_results, None).unwrap()).unwrap();

    let formatted_expected = format!(
        "{}\
| `sleep 0.1` | 105.7 ± 1.6 | 102.3 | 108.0 | 1.00 |
| `sleep 2` | 2005.0 ± 2.0 | 2002.0 | 2008.0 | 18.97 ± 0.29 |
",
        test_table_header("ms".to_string())
    );

    assert_eq!(formatted_expected, formatted);
}

/// This (again) demonstrates that the first entry's units (s) are used to set
/// the units for all entries when the time unit is not given.
#[test]
fn test_markdown_format_s() {
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

    let formatted = String::from_utf8(exporter.serialize(&timing_results, None).unwrap()).unwrap();

    let formatted_expected = format!(
        "{}\
| `sleep 2` | 2.005 ± 0.002 | 2.002 | 2.008 | 18.97 ± 0.29 |
| `sleep 0.1` | 0.106 ± 0.002 | 0.102 | 0.108 | 1.00 |
",
        test_table_header("s".to_string())
    );

    assert_eq!(formatted_expected, formatted);
}

/// The given time unit (s) is used to set the units for all entries.
#[test]
fn test_markdown_format_time_unit_s() {
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

    let formatted = String::from_utf8(
        exporter
            .serialize(&timing_results, Some(Unit::Second))
            .unwrap(),
    )
    .unwrap();

    let formatted_expected = format!(
        "{}\
| `sleep 0.1` | 0.106 ± 0.002 | 0.102 | 0.108 | 1.00 |
| `sleep 2` | 2.005 ± 0.002 | 2.002 | 2.008 | 18.97 ± 0.29 |
",
        test_table_header("s".to_string())
    );

    assert_eq!(formatted_expected, formatted);
}

/// This (again) demonstrates that the given time unit (ms) is used to set
/// the units for all entries.
#[test]
fn test_markdown_format_time_unit_ms() {
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

    let formatted = String::from_utf8(
        exporter
            .serialize(&timing_results, Some(Unit::MilliSecond))
            .unwrap(),
    )
    .unwrap();

    let formatted_expected = format!(
        "{}\
| `sleep 2` | 2005.0 ± 2.0 | 2002.0 | 2008.0 | 18.97 ± 0.29 |
| `sleep 0.1` | 105.7 ± 1.6 | 102.3 | 108.0 | 1.00 |
",
        test_table_header("ms".to_string())
    );

    assert_eq!(formatted_expected, formatted);
}
