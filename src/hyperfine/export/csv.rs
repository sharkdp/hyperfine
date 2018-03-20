use super::{ExportEntry, Exporter};

use csv::WriterBuilder;
use std::io::{Error, ErrorKind, Result};

pub struct CsvExporter {}

impl Exporter for CsvExporter {
    fn serialize(&self, results: &Vec<ExportEntry>) -> Result<Vec<u8>> {
        let mut writer = WriterBuilder::new().from_writer(vec![]);
        for res in results {
            writer.serialize(res)?;
        }

        if let Ok(inner) = writer.into_inner() {
            return Ok(inner);
        }
        Err(Error::new(ErrorKind::Other, "Error serializing to CSV"))
    }
}

impl CsvExporter {
    pub fn new() -> Self {
        CsvExporter {}
    }
}
