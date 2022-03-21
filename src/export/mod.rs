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
