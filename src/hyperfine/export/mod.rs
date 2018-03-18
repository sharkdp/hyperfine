mod csv;
mod json;

use self::csv::CsvExporter;
use self::json::JsonExporter;

use std::io::{Result, Write};
use std::fs::File;

use hyperfine::internal::Second;

/// The ExportEntry is the main set of values that will
/// be exported to files when requested.
#[derive(Debug, Default, Clone, Serialize)]
pub struct ExportEntry {
    /// The command that was run
    command: String,
    /// The mean run time
    mean: Second,
    /// The standard deviation of all run times
    stddev: Second,
    /// Time spend in user space
    user: Second,
    /// Time spent in system space
    system: Second,
}

impl ExportEntry {
    /// Create a new ExportEntry with the given values
    pub fn new(command: String, mean: f64, stddev: f64, user: f64, system: f64) -> Self {
        ExportEntry {
            command,
            mean,
            stddev,
            user,
            system,
        }
    }
}

/// The ResultExportType enum is used to denote the desired form
/// of exporter to use for a given file.
#[derive(Clone)]
pub enum ResultExportType {
    /// Export to a csv file with the provided name
    Csv(String),
    /// Export to a JSON file
    Json(String),
}

/// A ResultExporter is responsible for writing all results to the
/// appropriate file
trait ResultExporter {
    /// Write all entries to the target destination
    fn write(&self, values: &Vec<ExportEntry>) -> Result<Vec<u8>>;
}

/// Create a new ExportManager
pub fn create_export_manager<'a>() -> ExportManager {
    ExportManager {
        exporters: Vec::new(),
    }
}

/// The Exporter is the internal implementation of the ExportManager
pub struct ExportManager {
    exporters: Vec<ResultExportType>,
}

impl ExportManager {
    pub fn add_exporter<'a>(&mut self, for_type: ResultExportType) {
        self.exporters.push(for_type);
    }

    pub fn write_results(&self, to_write: Vec<ExportEntry>) -> Result<()> {
        for exp in &self.exporters {
            match exp {
                &ResultExportType::Csv(ref file) => {
                    let exp = CsvExporter::new();
                    let contents = exp.write(&to_write)?;
                    write_to_file(file, contents)?;
                }
                &ResultExportType::Json(ref file) => {
                    let exp = JsonExporter::new();
                    let contents = exp.write(&to_write)?;
                    write_to_file(file, contents)?;
                }
            }
        }
        Ok(())
    }
}

fn write_to_file(filename: &str, content: Vec<u8>) -> Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(&content)?;
    Ok(())
}
