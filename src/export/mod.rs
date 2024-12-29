use std::fs::{File, OpenOptions};
use std::io::Write;

mod asciidoc;
mod csv;
mod json;
mod markdown;
mod markup;
mod orgmode;
#[cfg(test)]
mod tests;

use self::asciidoc::AsciidocExporter;
use self::csv::CsvExporter;
use self::json::JsonExporter;
use self::markdown::MarkdownExporter;
use self::orgmode::OrgmodeExporter;

use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::options::SortOrder;
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
    fn serialize(
        &self,
        results: &[BenchmarkResult],
        unit: Option<Unit>,
        sort_order: SortOrder,
    ) -> Result<Vec<u8>>;
}

pub enum ExportTarget {
    File(String),
    Stdout,
}

struct ExporterWithTarget {
    exporter: Box<dyn Exporter>,
    target: ExportTarget,
}

/// Handles the management of multiple file exporters.
pub struct ExportManager {
    exporters: Vec<ExporterWithTarget>,
    time_unit: Option<Unit>,
    sort_order: SortOrder,
}

impl ExportManager {
    /// Build the ExportManager that will export the results specified
    /// in the given ArgMatches
    pub fn from_cli_arguments(
        matches: &ArgMatches,
        time_unit: Option<Unit>,
        sort_order: SortOrder,
    ) -> Result<Self> {
        let mut export_manager = Self {
            exporters: vec![],
            time_unit,
            sort_order,
        };
        {
            let mut add_exporter = |flag, exporttype| -> Result<()> {
                if let Some(filename) = matches.get_one::<String>(flag) {
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
        let exporter: Box<dyn Exporter> = match export_type {
            ExportType::Asciidoc => Box::<AsciidocExporter>::default(),
            ExportType::Csv => Box::<CsvExporter>::default(),
            ExportType::Json => Box::<JsonExporter>::default(),
            ExportType::Markdown => Box::<MarkdownExporter>::default(),
            ExportType::Orgmode => Box::<OrgmodeExporter>::default(),
        };

        self.exporters.push(ExporterWithTarget {
            exporter,
            target: if filename == "-" {
                ExportTarget::Stdout
            } else {
                let _ = File::create(filename)
                    .with_context(|| format!("Could not create export file '{filename}'"))?;
                ExportTarget::File(filename.to_string())
            },
        });

        Ok(())
    }

    /// Write the given results to all Exporters. The 'intermediate' flag specifies
    /// whether this is being called while still performing benchmarks, or if this
    /// is the final call after all benchmarks have been finished. In the former case,
    /// results are written to all file targets (to always have them up to date, even
    /// if a benchmark fails). In the latter case, we only print to stdout targets (in
    /// order not to clutter the output of hyperfine with intermediate results).
    pub fn write_results(&self, results: &[BenchmarkResult], intermediate: bool) -> Result<()> {
        for e in &self.exporters {
            let content = || {
                e.exporter
                    .serialize(results, self.time_unit, self.sort_order)
            };

            match e.target {
                ExportTarget::File(ref filename) => {
                    if intermediate {
                        write_to_file(filename, &content()?)?
                    }
                }
                ExportTarget::Stdout => {
                    if !intermediate {
                        println!();
                        println!("{}", String::from_utf8(content()?).unwrap());
                    }
                }
            }
        }
        Ok(())
    }
}

/// Write the given content to a file with the specified name
fn write_to_file(filename: &str, content: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().write(true).open(filename)?;
    file.write_all(content)
        .with_context(|| format!("Failed to export results to '{filename}'"))
}
