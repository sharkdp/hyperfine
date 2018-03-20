use super::{ExportEntry, Exporter};

use std::io::{Error, ErrorKind, Result};

use csv::WriterBuilder;

#[derive(Default)]
pub struct CsvExporter {}

impl Exporter for CsvExporter {
    fn serialize(&self, results: &Vec<ExportEntry>) -> Result<Vec<u8>> {
        let mut writer = WriterBuilder::new().from_writer(vec![]);
        for res in results {
            // The list of times can not be exported to the CSV file - remove it:
            let mut result = res.clone();
            result.times = None;

            writer.serialize(result)?;
        }

        writer
            .into_inner()
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }
}
