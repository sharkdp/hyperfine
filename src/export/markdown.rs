use std::io::{Error, ErrorKind, Result};

use super::Exporter;
use crate::benchmark_result::BenchmarkResult;
use crate::format::format_duration_value;
use crate::relative_speed::{self, BenchmarkResultWithRelativeSpeed};
use crate::units::Unit;

#[derive(Default)]
pub struct MarkdownExporter {}

impl Exporter for MarkdownExporter {
    fn serialize(&self, results: &[BenchmarkResult], unit: Option<Unit>) -> Result<Vec<u8>> {
        let unit = if let Some(unit) = unit {
            // Use the given unit for all entries.
            unit
        } else if let Some(first_result) = results.first() {
            // Use the first BenchmarkResult entry to determine the unit for all entries.
            format_duration_value(first_result.mean, None).1
        } else {
            // Default to `Second`.
            Unit::Second
        };

        if let Some(annotated_results) = relative_speed::compute(results) {
            let mut destination = start_table(unit);

            for result in annotated_results {
                add_table_row(&mut destination, &result, unit);
            }

            Ok(destination)
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Relative speed comparison is not available for Markdown export.",
            ))
        }
    }
}

fn table_header(unit_short_name: String) -> String {
    format!(
        "| Command | Mean [{unit}] | Min [{unit}] | Max [{unit}] | Relative |\n|:---|---:|---:|---:|---:|\n",
        unit = unit_short_name
    )
}

fn start_table(unit: Unit) -> Vec<u8> {
    table_header(unit.short_name()).bytes().collect()
}

fn add_table_row(dest: &mut Vec<u8>, entry: &BenchmarkResultWithRelativeSpeed, unit: Unit) {
    let result = &entry.result;
    let mean_str = format_duration_value(result.mean, Some(unit)).0;
    let stddev_str = format_duration_value(result.stddev, Some(unit)).0;
    let min_str = format_duration_value(result.min, Some(unit)).0;
    let max_str = format_duration_value(result.max, Some(unit)).0;
    let rel_str = format!("{:.2}", entry.relative_speed);
    let rel_stddev_str = if entry.is_fastest {
        "".into()
    } else {
        format!(" ± {:.2}", entry.relative_speed_stddev)
    };

    dest.extend(
        format!(
            "| `{command}` | {mean} ± {stddev} | {min} | {max} | {rel}{rel_stddev} |\n",
            command = result.command.replace("|", "\\|"),
            mean = mean_str,
            stddev = stddev_str,
            min = min_str,
            max = max_str,
            rel = rel_str,
            rel_stddev = rel_stddev_str,
        )
        .as_bytes(),
    );
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

    let mut timing_results = vec![];

    timing_results.push(BenchmarkResult::new(
        String::from("sleep 0.1"),
        0.1057,                          // mean
        0.0016,                          // std dev
        0.1057,                          // median
        0.0009,                          // user_mean
        0.0011,                          // system_mean
        0.1023,                          // min
        0.1080,                          // max
        vec![0.1, 0.1, 0.1],             // times
        vec![Some(0), Some(0), Some(0)], // exit codes
        BTreeMap::new(),                 // parameter
    ));

    timing_results.push(BenchmarkResult::new(
        String::from("sleep 2"),
        2.0050,                          // mean
        0.0020,                          // std dev
        2.0050,                          // median
        0.0009,                          // user_mean
        0.0012,                          // system_mean
        2.0020,                          // min
        2.0080,                          // max
        vec![2.0, 2.0, 2.0],             // times
        vec![Some(0), Some(0), Some(0)], // exit codes
        BTreeMap::new(),                 // parameter
    ));

    let formatted = String::from_utf8(exporter.serialize(&timing_results, None).unwrap()).unwrap();

    let formatted_expected = format!(
        "{}\
| `sleep 0.1` | 105.7 ± 1.6 | 102.3 | 108.0 | 1.00 |
| `sleep 2` | 2005.0 ± 2.0 | 2002.0 | 2008.0 | 18.97 ± 0.29 |
",
        table_header("ms".to_string())
    );

    assert_eq!(formatted_expected, formatted);
}

/// This (again) demonstrates that the first entry's units (s) are used to set
/// the units for all entries when the time unit is not given.
#[test]
fn test_markdown_format_s() {
    use std::collections::BTreeMap;
    let exporter = MarkdownExporter::default();

    let mut timing_results = vec![];

    timing_results.push(BenchmarkResult::new(
        String::from("sleep 2"),
        2.0050,                          // mean
        0.0020,                          // std dev
        2.0050,                          // median
        0.0009,                          // user_mean
        0.0012,                          // system_mean
        2.0020,                          // min
        2.0080,                          // max
        vec![2.0, 2.0, 2.0],             // times
        vec![Some(0), Some(0), Some(0)], // exit codes
        BTreeMap::new(),                 // parameter
    ));

    timing_results.push(BenchmarkResult::new(
        String::from("sleep 0.1"),
        0.1057,                          // mean
        0.0016,                          // std dev
        0.1057,                          // median
        0.0009,                          // user_mean
        0.0011,                          // system_mean
        0.1023,                          // min
        0.1080,                          // max
        vec![0.1, 0.1, 0.1],             // times
        vec![Some(0), Some(0), Some(0)], // exit codes
        BTreeMap::new(),                 // parameter
    ));

    let formatted = String::from_utf8(exporter.serialize(&timing_results, None).unwrap()).unwrap();

    let formatted_expected = format!(
        "{}\
| `sleep 2` | 2.005 ± 0.002 | 2.002 | 2.008 | 18.97 ± 0.29 |
| `sleep 0.1` | 0.106 ± 0.002 | 0.102 | 0.108 | 1.00 |
",
        table_header("s".to_string())
    );

    assert_eq!(formatted_expected, formatted);
}

/// The given time unit (s) is used to set the units for all entries.
#[test]
fn test_markdown_format_time_unit_s() {
    use std::collections::BTreeMap;
    let exporter = MarkdownExporter::default();

    let mut timing_results = vec![];

    timing_results.push(BenchmarkResult::new(
        String::from("sleep 0.1"),
        0.1057,                          // mean
        0.0016,                          // std dev
        0.1057,                          // median
        0.0009,                          // user_mean
        0.0011,                          // system_mean
        0.1023,                          // min
        0.1080,                          // max
        vec![0.1, 0.1, 0.1],             // times
        vec![Some(0), Some(0), Some(0)], // exit codes
        BTreeMap::new(),                 // parameter
    ));

    timing_results.push(BenchmarkResult::new(
        String::from("sleep 2"),
        2.0050,                          // mean
        0.0020,                          // std dev
        2.0050,                          // median
        0.0009,                          // user_mean
        0.0012,                          // system_mean
        2.0020,                          // min
        2.0080,                          // max
        vec![2.0, 2.0, 2.0],             // times
        vec![Some(0), Some(0), Some(0)], // exit codes
        BTreeMap::new(),                 // parameter
    ));

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
        table_header("s".to_string())
    );

    assert_eq!(formatted_expected, formatted);
}

/// This (again) demonstrates that the given time unit (ms) is used to set
/// the units for all entries.
#[test]
fn test_markdown_format_time_unit_ms() {
    use std::collections::BTreeMap;
    let exporter = MarkdownExporter::default();

    let mut timing_results = vec![];

    timing_results.push(BenchmarkResult::new(
        String::from("sleep 2"),
        2.0050,                          // mean
        0.0020,                          // std dev
        2.0050,                          // median
        0.0009,                          // user_mean
        0.0012,                          // system_mean
        2.0020,                          // min
        2.0080,                          // max
        vec![2.0, 2.0, 2.0],             // times
        vec![Some(0), Some(0), Some(0)], // exit codes
        BTreeMap::new(),                 // parameter
    ));

    timing_results.push(BenchmarkResult::new(
        String::from("sleep 0.1"),
        0.1057,                          // mean
        0.0016,                          // std dev
        0.1057,                          // median
        0.0009,                          // user_mean
        0.0011,                          // system_mean
        0.1023,                          // min
        0.1080,                          // max
        vec![0.1, 0.1, 0.1],             // times
        vec![Some(0), Some(0), Some(0)], // exit codes
        BTreeMap::new(),                 // parameter
    ));

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
        table_header("ms".to_string())
    );

    assert_eq!(formatted_expected, formatted);
}
