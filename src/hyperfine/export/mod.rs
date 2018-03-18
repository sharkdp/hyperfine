mod csv;

use self::csv::CsvExporter;

use std::io::Result;

/// The ExportEntry is the main set of values that will
/// be exported to files when requested.
#[derive(Debug, Default, Clone, Serialize)]
pub struct ExportEntry {
    /// The command that was run
    command: String,
    /// The mean run time
    mean: f64,
    /// The standard deviation of all run times
    stddev: f64,
    /// Time spend in user space
    user: f64,
    /// Time spent in system space
    system: f64,
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

/// The ExportManager handles coordination of multiple ResultExporters
pub trait ExportManager {
    /// Add a new exporter to this manager for the given type
    fn add_exporter(&mut self, for_type: &ResultExportType);

    /// Add a new result to all exporters contained in the manager
    fn add_result(&mut self, result: ExportEntry);

    /// Trigger writes from all exporters
    fn write_results(&self) -> Result<()>;
}

/// Create a new ExportManager
pub fn create_export_manager() -> Box<ExportManager> {
    Box::new(Exporter {
        exporters: Vec::new(),
        results: Vec::new(),
    })
}

/// The Exporter is the internal implementation of the ExportManager
struct Exporter {
    exporters: Vec<Box<ResultExporter>>,
    results: Vec<ExportEntry>,
}

impl ExportManager for Exporter {
    fn add_exporter(&mut self, for_type: &ResultExportType) {
        match for_type {
            &ResultExportType::Csv(ref file_name) => self.exporters
                .push(Box::from(CsvExporter::new(file_name.clone()))),
        };
    }

    fn add_result(&mut self, result: ExportEntry) {
        self.results.push(result);
    }

    fn write_results(&self) -> Result<()> {
        for exp in &self.exporters {
            exp.write(&self.results)?;
        }
        Ok(())
    }
}
