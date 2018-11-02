use super::Exporter;

use hyperfine::types::BenchmarkResult;
use hyperfine::units::Unit;

use std::io::{Error, ErrorKind, Result};

use csv::WriterBuilder;

#[derive(Default)]
pub struct CsvExporter {}

impl Exporter for CsvExporter {
    fn serialize(&self, results: &Vec<BenchmarkResult>, _unit: Option<Unit>) -> Result<Vec<u8>> {
        let mut writer = WriterBuilder::new().from_writer(vec![]);
        for res in results {
            // The list of times cannot be exported to the CSV file - remove it:
            let mut result = res.clone();
            result.times = None;

            writer.serialize(result)?;
        }

        writer
            .into_inner()
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }
}
