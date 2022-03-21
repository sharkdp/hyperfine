use std::fs::{File, OpenOptions};
use std::io::Write;

mod asciidoc;
mod csv;
mod json;
mod markdown;
mod markup;
mod orgmode;

use self::asciidoc::AsciidocExporter;
use self::csv::CsvExporter;
use self::json::JsonExporter;
use self::markdown::MarkdownExporter;
use self::orgmode::OrgmodeExporter;

use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::output::format::format_duration_value;
use crate::util::units::Unit;

use anyhow::{Context, Result};
use clap::ArgMatches;

/// The desired form of exporter to use for a given file.
#[derive(Clone)]
pub enum ExportType {
    /// Asciidoc Table
    Asciidoc,

    /// CSV (comma separated values) format
    Csv,

    /// JSON format
    Json,

    /// Markdown table
    Markdown,

    /// Emacs org-mode tables
    Orgmode,
}

/// Interface for different exporters.
trait Exporter {
    /// Export the given entries in the serialized form.
    fn serialize(&self, results: &[BenchmarkResult], unit: Option<Unit>) -> Result<Vec<u8>>;

    fn unit(&self, results: &[BenchmarkResult], unit: Option<Unit>) -> Unit {
        return if let Some(unit) = unit {
            // Use the given unit for all entries.
            unit
        } else if let Some(first_result) = results.first() {
            // Use the first BenchmarkResult entry to determine the unit for all entries.
            format_duration_value(first_result.mean, None).1
        } else {
            // Default to `Second`.
            Unit::Second
        };
    }
}

struct ExporterWithFilename {
    exporter: Box<dyn Exporter>,
    filename: String,
}

/// Handles the management of multiple file exporters.
#[derive(Default)]
pub struct ExportManager {
    exporters: Vec<ExporterWithFilename>,
}

impl ExportManager {
    /// Build the ExportManager that will export the results specified
    /// in the given ArgMatches
    pub fn from_cli_arguments(matches: &ArgMatches) -> Result<Self> {
        let mut export_manager = Self::default();
        {
            let mut add_exporter = |flag, exporttype| -> Result<()> {
                if let Some(filename) = matches.value_of(flag) {
                    export_manager.add_exporter(exporttype, filename)?;
                }
                Ok(())
            };
            add_exporter("export-asciidoc", ExportType::Asciidoc)?;
            add_exporter("export-json", ExportType::Json)?;
            add_exporter("export-csv", ExportType::Csv)?;
            add_exporter("export-markdown", ExportType::Markdown)?;
            add_exporter("export-orgmode", ExportType::Orgmode)?;
        }
        Ok(export_manager)
    }

    /// Add an additional exporter to the ExportManager
    pub fn add_exporter(&mut self, export_type: ExportType, filename: &str) -> Result<()> {
        let _ = File::create(filename)
            .with_context(|| format!("Could not create export file '{}'", filename))?;

        let exporter: Box<dyn Exporter> = match export_type {
            ExportType::Asciidoc => Box::new(AsciidocExporter::default()),
            ExportType::Csv => Box::new(CsvExporter::default()),
            ExportType::Json => Box::new(JsonExporter::default()),
            ExportType::Markdown => Box::new(MarkdownExporter::default()),
            ExportType::Orgmode => Box::new(OrgmodeExporter::default()),
        };
        self.exporters.push(ExporterWithFilename {
            exporter,
            filename: filename.to_string(),
        });

        Ok(())
    }

    /// Write the given results to all Exporters contained within this manager
    pub fn write_results(&self, results: &[BenchmarkResult], unit: Option<Unit>) -> Result<()> {
        for e in &self.exporters {
            let file_content = e.exporter.serialize(results, unit)?;
            write_to_file(&e.filename, &file_content)?;
        }
        Ok(())
    }
}

/// Write the given content to a file with the specified name
fn write_to_file(filename: &str, content: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().write(true).open(filename)?;
    file.write_all(content)
        .with_context(|| format!("Failed to export results to '{}'", filename))
}

#[cfg(test)]
#[derive(Default)]
struct TestExporter;

#[cfg(test)]
impl Exporter for TestExporter {
    fn serialize(&self, _results: &[BenchmarkResult], _unit: Option<Unit>) -> Result<Vec<u8>> {
        assert_eq!(
            "",
            "This 'Exporter' trait implementation shall only be used to test the 'unit' function!"
        );
        Ok(vec![])
    }
}

/// Check unit resolving for timing results and given unit 's'
#[test]
fn test_markup_table_unit_given_s() {
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

    let exporter = TestExporter::default();
    let markup_actual = exporter.unit(&results, unit);
    let markup_expected = Unit::Second;

    assert_eq!(markup_expected, markup_actual);
}

/// Check unit resolving for timing results and given unit 'ms'
#[test]
fn test_markup_table_unit_given_ms() {
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

    let exporter = TestExporter::default();
    let markup_actual = exporter.unit(&results, unit);
    let markup_expected = Unit::MilliSecond;

    assert_eq!(markup_expected, markup_actual);
}

/// Check unit resolving for timing results using the first result entry as 's'
#[test]
fn test_markup_table_unit_first_s() {
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

    let exporter = TestExporter::default();
    let markup_actual = exporter.unit(&results, unit);
    let markup_expected = Unit::Second;

    assert_eq!(markup_expected, markup_actual);
}

/// Check unit resolving for timing results using the first result entry as 'ms'
#[test]
fn test_markup_table_unit_first_ms() {
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

    let exporter = TestExporter::default();
    let markup_actual = exporter.unit(&results, unit);
    let markup_expected = Unit::MilliSecond;

    assert_eq!(markup_expected, markup_actual);
}

/// Check unit resolving for not timing results and no given unit defaulting to 's'
#[test]
fn test_markup_table_unit_default_s() {
    let results: Vec<BenchmarkResult> = vec![];
    let unit = None;

    let exporter = TestExporter::default();
    let markup_actual = exporter.unit(&results, unit);
    let markup_expected = Unit::Second;

    assert_eq!(markup_expected, markup_actual);
}
