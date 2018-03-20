use super::{ExportEntry, Exporter};

use csv::WriterBuilder;
use std::io::{Error, ErrorKind, Result};

pub struct CsvExporter {}

impl Exporter for CsvExporter {
    fn serialize(&self, results: &Vec<ExportEntry>) -> Result<String> {
        let mut writer = WriterBuilder::new().from_writer(vec![]);
        for res in results {
            writer.serialize(res)?;
        }

        writer
            .into_inner()
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .ok_or(Error::new(ErrorKind::Other, "Error serializing to CSV"))
    }
}

impl CsvExporter {
    pub fn new() -> Self {
        CsvExporter {}
    }
}
