use super::{ExportEntry, Exporter};

use std::io::{Error, ErrorKind, Result};

use csv::WriterBuilder;

use hyperfine::internal::Second;

/// Set of values that will be exported.
#[derive(Debug, Serialize)]
pub struct CsvEntry {
    command: String,
    mean: Second,
    stddev: Second,
    user: Second,
    system: Second,
    min: Second,
    max: Second,
}


#[derive(Default)]
pub struct CsvExporter {}

impl Exporter for CsvExporter {
    fn serialize(&self, results: &Vec<ExportEntry>) -> Result<Vec<u8>> {
        let mut writer = WriterBuilder::new().from_writer(vec![]);
        for res in results {
            writer.serialize(&CsvEntry {
                command: res.command.clone(),
                mean: res.mean,
                stddev: res.stddev,
                user: res.user,
                system: res.system,
                min: res.min,
                max: res.max,
            })?;
        }

        writer
            .into_inner()
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }
}
