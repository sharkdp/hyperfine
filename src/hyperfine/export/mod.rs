mod asciidoc;
mod csv;
mod json;
mod markdown;

use self::asciidoc::AsciidocExporter;
use self::csv::CsvExporter;
use self::json::JsonExporter;
use self::markdown::MarkdownExporter;

use std::fs::File;
use std::io::{Result, Write};

use crate::hyperfine::types::BenchmarkResult;
use crate::hyperfine::units::Unit;

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
}

/// Interface for different exporters.
trait Exporter {
    /// Export the given entries in the serialized form.
    fn serialize(&self, results: &[BenchmarkResult], unit: Option<Unit>) -> Result<Vec<u8>>;
    fn write_to_file_incremental(&mut self, result: &BenchmarkResult, unit: Option<Unit>) -> Result<()>;
}

struct ExporterWithFilename {
    exporter: Box<dyn Exporter>,
    filename: String,
}

/// Handles the management of multiple file exporters.
pub struct ExportManager {
    exporters: Vec<ExporterWithFilename>,
}

impl ExportManager {
    /// Create a new ExportManager
    pub fn new() -> ExportManager {
        ExportManager {
            exporters: Vec::new(),
        }
    }

    /// Add an additional exporter to the ExportManager
    pub fn add_exporter(&mut self, export_type: ExportType, filename: &str) {
        let exporter: Box<dyn Exporter> = match export_type {
            ExportType::Asciidoc => Box::new(AsciidocExporter::default()),
            ExportType::Csv => Box::new(CsvExporter::new(filename)),
            ExportType::Json => Box::new(JsonExporter::default()),
            ExportType::Markdown => Box::new(MarkdownExporter::default()),
        };
        self.exporters.push(ExporterWithFilename {
            exporter,
            filename: filename.to_string(),
        });
    }

    /// Write the given results to all Exporters contained within this manager
    pub fn write_results(&self, results: Vec<BenchmarkResult>, unit: Option<Unit>) -> Result<()> {
        for e in &self.exporters {
            let file_content = e.exporter.serialize(&results, unit)?;
            write_to_file(&e.filename, &file_content)?;
        }
        Ok(())
    }

    pub fn write_incremental_results(&mut self, result: &BenchmarkResult, unit: Option<Unit>) -> Result<()> { 
        for e in &mut self.exporters {
            e.exporter.write_to_file_incremental(result, unit)?;}
        Ok(())
    }
}

/// Write the given content to a file with the specified name
fn write_to_file(filename: &str, content: &[u8]) -> Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(content)
}
