use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::benchmark::relative_speed;
use crate::export::markup::MarkupFormatter;
use crate::util::units::Unit;

use anyhow::{anyhow, Result};

#[derive(Default)]
pub struct OrgmodeFormatter;

impl MarkupFormatter for OrgmodeFormatter {
    fn table_data(&self, data: &[&str]) -> String {
        format!(
            "| {}  |  {} |\n",
            data.first().unwrap(),
            &data[1..].join(" |  ")
        )
    }

    fn table_line(&self, size: usize) -> String {
        format!("|{}--|\n", "--+".repeat(size - 1))
    }

    fn command(&self, cmd: &str) -> String {
        format!("={}=", cmd)
    }
}

#[derive(Default)]
pub struct OrgmodeExporter {}

impl Exporter for OrgmodeExporter {
    fn serialize(&self, results: &[BenchmarkResult], unit: Option<Unit>) -> Result<Vec<u8>> {
        let unit = self.unit(results, unit);
        let entries = relative_speed::compute(results);
        if entries.is_none() {
            return Err(anyhow!(
                "Relative speed comparison is not available for Emacs org-mode export."
            ));
        }

        let formatter = OrgmodeFormatter::default();
        let table = formatter.table_results(&entries.unwrap(), unit);
        Ok(table.as_bytes().to_vec())
    }
}

/// Check Emacs org-mode data row formatting
#[test]
fn test_orgmode_formatter_table_data() {
    let formatter = OrgmodeFormatter::default();
    let data = vec!["a", "b", "c"];

    let actual = formatter.table_data(&data);
    let expect = "| a  |  b |  c |\n";

    assert_eq!(expect, actual);
}

/// Check Emacs org-mode horizontal line formatting
#[test]
fn test_orgmode_formatter_table_line() {
    let formatter = OrgmodeFormatter::default();
    let size = 5;

    let actual = formatter.table_line(size);
    let expect = "|--+--+--+--+--|\n";

    assert_eq!(expect, actual);
}

/// Test helper function to create unit-based header and horizontal line
/// independently from the markup functionality for Emacs org-mode.
#[cfg(test)]
fn cfg_test_table_header(unit_short_name: String) -> String {
    format!(
        "| Command  |  Mean [{unit}] |  Min [{unit}] |  Max [{unit}] |  Relative |\n|--+--+--+--+--|\n",
        unit = unit_short_name
    )
}

/// Ensure the Emacs org-mode output includes the table header and the multiple
/// benchmark results as a table. The list of actual times is not included
/// in the output.
///
/// This also demonstrates that the first entry's units (ms) are used to set
/// the units for all entries when the time unit is not given.
#[test]
fn test_orgmode_format_ms() {
    use std::collections::BTreeMap;
    let exporter = OrgmodeExporter::default();

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

    let actual = String::from_utf8(exporter.serialize(&results, None).unwrap()).unwrap();
    let expect = format!(
        "{}\
| =sleep 0.1=  |  105.7 ± 1.6 |  102.3 |  108.0 |  1.00 |
| =sleep 2=  |  2005.0 ± 2.0 |  2002.0 |  2008.0 |  18.97 ± 0.29 |
",
        cfg_test_table_header("ms".to_string())
    );

    assert_eq!(expect, actual);
}

/// This test demonstrates that the given unit (s) is used to set
/// the units for all entries.
#[test]
fn test_orgmode_format_s() {
    use std::collections::BTreeMap;
    let exporter = OrgmodeExporter::default();

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

    let actual =
        String::from_utf8(exporter.serialize(&results, Some(Unit::Second)).unwrap()).unwrap();
    let expect = format!(
        "{}\
| =sleep 2=  |  2.005 ± 0.002 |  2.002 |  2.008 |  18.97 ± 0.29 |
| =sleep 0.1=  |  0.106 ± 0.002 |  0.102 |  0.108 |  1.00 |
",
        cfg_test_table_header("s".to_string())
    );

    assert_eq!(expect, actual);
}
