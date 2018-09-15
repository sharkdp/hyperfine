use super::Exporter;

use hyperfine::format::{Unit, format_duration_value};
use hyperfine::types::BenchmarkResult;

use std::io::Result;

macro_rules! TABLE_HEADER {
    ($unit:expr) => {
        format!("| Command | Mean [{unit}] | Min…Max [{unit}] |\n|:---|---:|---:|\n", unit=$unit) };
}

#[derive(Default)]
pub struct MarkdownExporter {}

impl Exporter for MarkdownExporter {
    fn serialize(&self, results: &Vec<BenchmarkResult>) -> Result<Vec<u8>> {
        // Default to `Second`.
        let mut unit = Unit::Second;

        if !results.is_empty() {
            // Use the first BenchmarkResult entry to determine the unit for all entries.
            unit = format_duration_value(results[0].mean, None).1;
        }

        let mut destination = start_table(unit);

        for result in results {
            add_table_row(&mut destination, result, unit);
        }

        Ok(destination)
    }
}

fn start_table(unit: Unit) -> Vec<u8> {
    TABLE_HEADER!(unit.short_name()).bytes().collect()
}

fn add_table_row(dest: &mut Vec<u8>, entry: &BenchmarkResult, unit: Unit) {
    let mean_str = format_duration_value(entry.mean, Some(unit)).0;
    let stddev_str = format_duration_value(entry.stddev, Some(unit)).0;
    let min_str = format_duration_value(entry.min, Some(unit)).0;
    let max_str = format_duration_value(entry.max, Some(unit)).0;

    dest.extend(
        format!(
            "| `{command}` | {mean} ± {stddev} | {min}…{max} |\n",
            command=entry.command.replace("|", "\\|"),
            mean=mean_str,
            stddev=stddev_str,
            min=min_str,
            max=max_str,
        ).as_bytes(),
    );
}

/// Ensure the markdown output includes the table header and the multiple
/// benchmark results as a table. The list of actual times is not included
/// in the output.
///
/// This also demonstrates that the first entry's units (ms) are used to set
/// the units for all entries.
#[test]
fn test_markdown_format_ms() {
    let exporter = MarkdownExporter::default();

    let mut timing_results = vec![];

    timing_results.push(BenchmarkResult::new(
            String::from("sleep 0.1"),
            0.1057, // mean
            0.0016, // std dev
            0.0009, // user_mean
            0.0011, // system_mean
            0.1023, // min
            0.1080, // max
            vec![0.1, 0.1, 0.1], // times
            ));

    timing_results.push(BenchmarkResult::new(
            String::from("sleep 2"),
            2.0050, // mean
            0.0020, // std dev
            0.0009, // user_mean
            0.0012, // system_mean
            2.0020, // min
            2.0080, // max
            vec![2.0, 2.0, 2.0], // times
            ));

    let formatted = String::from_utf8(exporter.serialize(&timing_results).unwrap()).unwrap();

    let formatted_expected = format!(
"{}\
| `sleep 0.1` | 105.7 ± 1.6 | 102.3…108.0 |
| `sleep 2` | 2005.0 ± 2.0 | 2002.0…2008.0 |
", TABLE_HEADER!("ms"));

    assert_eq!(formatted_expected, formatted);
}

/// This (again) demonstrates that the first entry's units (s) are used to set
/// the units for all entries.
#[test]
fn test_markdown_format_s() {
    let exporter = MarkdownExporter::default();

    let mut timing_results = vec![];

    timing_results.push(BenchmarkResult::new(
            String::from("sleep 2"),
            2.0050, // mean
            0.0020, // std dev
            0.0009, // user_mean
            0.0012, // system_mean
            2.0020, // min
            2.0080, // max
            vec![2.0, 2.0, 2.0], // times
            ));

    timing_results.push(BenchmarkResult::new(
            String::from("sleep 0.1"),
            0.1057, // mean
            0.0016, // std dev
            0.0009, // user_mean
            0.0011, // system_mean
            0.1023, // min
            0.1080, // max
            vec![0.1, 0.1, 0.1], // times
            ));

    let formatted = String::from_utf8(exporter.serialize(&timing_results).unwrap()).unwrap();

    let formatted_expected = format!(
"{}\
| `sleep 2` | 2.005 ± 0.002 | 2.002…2.008 |
| `sleep 0.1` | 0.106 ± 0.002 | 0.102…0.108 |
", TABLE_HEADER!("s"));

    assert_eq!(formatted_expected, formatted);
}

/// An empty list of benchmark results will only include the table header
/// in the markdown output, using the default `Seconds` unit.
#[test]
fn test_markdown_format_empty_results() {
    let exporter = MarkdownExporter::default();

    let timing_results = vec![];

    let formatted = String::from_utf8(exporter.serialize(&timing_results).unwrap()).unwrap();

    let formatted_expected = TABLE_HEADER!("s");

    assert_eq!(formatted_expected, formatted);
}
