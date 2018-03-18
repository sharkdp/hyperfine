mod csv;

use self::csv::CsvExporter;

use std::io::Result;

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
pub enum ResultExportType {
    /// Export to a csv file with the provided name
    Csv(String),
}

/// A ResultExporter is responsible for writing all results to the
/// appropriate file
trait ResultExporter {
    /// Write all entries to the target destination
    fn write(&self, values: &Vec<ExportEntry>) -> Result<()>;
}

/// Create a new ExportManager
pub fn create_export_manager() -> ExportManager {
    ExportManager {
        exporters: Vec::new(),
    }
}

/// The Exporter is the internal implementation of the ExportManager
pub struct ExportManager {
    exporters: Vec<Box<ResultExporter>>,
}

impl ExportManager {
    pub fn add_exporter(&mut self, for_type: &ResultExportType) {
        match for_type {
            &ResultExportType::Csv(ref file_name) => self.exporters
                .push(Box::from(CsvExporter::new(file_name.clone()))),
        };
    }

    pub fn write_results(&self, to_write: Vec<ExportEntry>) -> Result<()> {
        for exp in &self.exporters {
            exp.write(&to_write)?;
        }
        Ok(())
    }
}
