use crate::benchmark::relative_speed::BenchmarkResultWithRelativeSpeed;
use crate::benchmark::{benchmark_result::BenchmarkResult, relative_speed};
use crate::output::format::format_duration_value;
use crate::util::units::Unit;

use super::Exporter;
use anyhow::{anyhow, Result};

pub enum Alignment {
    Left,
    Right,
}
pub trait MarkupExporter {
    fn table_results(&self, entries: &[BenchmarkResultWithRelativeSpeed], unit: Unit) -> String {
        // prepare table header strings
        let notation = format!("[{}]", unit.short_name());

        // prepare table cells alignment
        let cells_alignment = [
            Alignment::Left,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
        ];

        // emit table header format
        let mut table = self.table_header(&cells_alignment);

        // emit table header data
        table.push_str(&self.table_row(&[
            "Command",
            &format!("Mean {}", notation),
            &format!("Min {}", notation),
            &format!("Max {}", notation),
            &format!("User {}", notation),
            &format!("System {}", notation),
            "Relative",
        ]));

        // emit horizontal line
        table.push_str(&self.table_divider(&cells_alignment));

        for entry in entries {
            let measurement = &entry.result;
            // prepare data row strings
            let cmd_str = measurement.command.replace('|', "\\|");
            let mean_str = format_duration_value(measurement.mean, Some(unit)).0;
            let stddev_str = if let Some(stddev) = measurement.stddev {
                format!(" ± {}", format_duration_value(stddev, Some(unit)).0)
            } else {
                "".into()
            };
            let min_str = format_duration_value(measurement.min, Some(unit)).0;
            let max_str = format_duration_value(measurement.max, Some(unit)).0;
            let user_str = format_duration_value(measurement.user, Some(unit)).0;
            let system_str = format_duration_value(measurement.system, Some(unit)).0;
            let rel_str = format!("{:.2}", entry.relative_speed);
            let rel_stddev_str = if entry.is_fastest {
                "".into()
            } else if let Some(stddev) = entry.relative_speed_stddev {
                format!(" ± {:.2}", stddev)
            } else {
                "".into()
            };

            // prepare table row entries
            table.push_str(&self.table_row(&[
                &self.command(&cmd_str),
                &format!("{}{}", mean_str, stddev_str),
                &min_str,
                &max_str,
                &user_str,
                &system_str,
                &format!("{}{}", rel_str, rel_stddev_str),
            ]))
        }

        // emit table footer format
        table.push_str(&self.table_footer(&cells_alignment));

        table
    }

    fn table_row(&self, cells: &[&str]) -> String;

    fn table_divider(&self, cell_aligmnents: &[Alignment]) -> String;

    fn table_header(&self, _cell_aligmnents: &[Alignment]) -> String {
        "".to_string()
    }

    fn table_footer(&self, _cell_aligmnents: &[Alignment]) -> String {
        "".to_string()
    }

    fn command(&self, size: &str) -> String;
}

fn determine_unit_from_results(results: &[BenchmarkResult]) -> Unit {
    if let Some(first_result) = results.first() {
        // Use the first BenchmarkResult entry to determine the unit for all entries.
        format_duration_value(first_result.mean, None).1
    } else {
        // Default to `Second`.
        Unit::Second
    }
}

impl<T: MarkupExporter> Exporter for T {
    fn serialize(&self, results: &[BenchmarkResult], unit: Option<Unit>) -> Result<Vec<u8>> {
        let unit = unit.unwrap_or_else(|| determine_unit_from_results(results));
        let entries = relative_speed::compute(results);
        if entries.is_none() {
            return Err(anyhow!(
                "Relative speed comparison is not available for markup exporter."
            ));
        }

        let table = self.table_results(&entries.unwrap(), unit);
        Ok(table.as_bytes().to_vec())
    }
}

/// Check unit resolving for timing results and given unit 's'
#[test]
fn test_determine_unit_from_results_unit_given_s() {
    use std::collections::BTreeMap;
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
    let unit = Some(Unit::Second);

    let markup_actual = unit.unwrap_or_else(|| determine_unit_from_results(&results));
    let markup_expected = Unit::Second;

    assert_eq!(markup_expected, markup_actual);
}

/// Check unit resolving for timing results and given unit 'ms'
#[test]
fn test_determine_unit_from_results_unit_given_ms() {
    use std::collections::BTreeMap;
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
    let unit = Some(Unit::MilliSecond);

    let markup_actual = unit.unwrap_or_else(|| determine_unit_from_results(&results));
    let markup_expected = Unit::MilliSecond;

    assert_eq!(markup_expected, markup_actual);
}

/// Check unit resolving for timing results using the first result entry as 's'
#[test]
fn test_determine_unit_from_results_unit_first_s() {
    use std::collections::BTreeMap;
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
    let unit = None;

    let markup_actual = unit.unwrap_or_else(|| determine_unit_from_results(&results));
    let markup_expected = Unit::Second;

    assert_eq!(markup_expected, markup_actual);
}

/// Check unit resolving for timing results using the first result entry as 'ms'
#[test]
fn test_determine_unit_from_results_unit_first_ms() {
    use std::collections::BTreeMap;
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
    let unit = None;

    let markup_actual = unit.unwrap_or_else(|| determine_unit_from_results(&results));
    let markup_expected = Unit::MilliSecond;

    assert_eq!(markup_expected, markup_actual);
}

/// Check unit resolving for not timing results and no given unit defaulting to 's'
#[test]
fn test_determine_unit_from_results_unit_default_s() {
    let results: Vec<BenchmarkResult> = vec![];
    let unit = None;

    let markup_actual = unit.unwrap_or_else(|| determine_unit_from_results(&results));
    let markup_expected = Unit::Second;

    assert_eq!(markup_expected, markup_actual);
}
