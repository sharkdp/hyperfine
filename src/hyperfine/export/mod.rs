mod csv;
mod json;

use self::csv::CsvExporter;
use self::json::JsonExporter;

use std::io::{Result, Write};
use std::fs::File;

use hyperfine::internal::Second;

/// Set of values that will be exported.
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

    /// Min time measured
    min: Second,

    /// Max time measured
    max: Second,
}

impl ExportEntry {
    /// Create a new entry with the given values.
    pub fn new(
        command: String,
        mean: Second,
        stddev: Second,
        user: Second,
        system: Second,
        min: Second,
        max: Second,
    ) -> Self {
        ExportEntry {
            command,
            mean,
            stddev,
            user,
            system,
            min,
            max,
        }
    }
}

/// The desired form of exporter to use for a given file.
#[derive(Clone)]
pub enum ExportType {
    /// CSV (comma separated values) format
    Csv(String),

    /// JSON format
    Json(String),
}

/// Interface for different exporters.
trait Exporter {
    /// Export the given entries in the serialized form.
    fn serialize(&self, results: &Vec<ExportEntry>) -> Result<String>;
}

/// Handles the management of multiple file exporters.
pub struct ExportManager {
    exporters: Vec<ExportType>,
}

impl ExportManager {
    /// Create a new ExportManager
    pub fn new() -> ExportManager {
        ExportManager {
            exporters: Vec::new(),
        }
    }

    /// Add an additional exporter to the ExportManager
    pub fn add_exporter(&mut self, for_type: ExportType) {
        self.exporters.push(for_type);
    }

    /// Write the given results to all Exporters contained within this manager
    pub fn write_results(&self, to_write: Vec<ExportEntry>) -> Result<()> {
        for exp in &self.exporters {
            let (exporter, filename): (Box<Exporter>, &str) = match exp {
                &ExportType::Csv(ref file) => (Box::from(CsvExporter::new()), file),
                &ExportType::Json(ref file) => (Box::from(JsonExporter::new()), file),
            };

            let file_content = exporter.serialize(&to_write)?;
            write_to_file(filename, file_content)?;
        }
        Ok(())
    }
}

/// Write the given content to a file with the specified name
fn write_to_file(filename: &str, content: String) -> Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
