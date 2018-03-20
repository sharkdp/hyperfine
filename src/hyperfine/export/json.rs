use super::{ExportEntry, Exporter};

use std::io::{Error, ErrorKind, Result};

use serde_json::to_vec_pretty;

#[derive(Serialize, Debug)]
struct HyperfineSummary<'a> {
    results: &'a Vec<ExportEntry>,
}

#[derive(Default)]
pub struct JsonExporter {}

impl Exporter for JsonExporter {
    fn serialize(&self, results: &Vec<ExportEntry>) -> Result<Vec<u8>> {
        to_vec_pretty(&HyperfineSummary { results }).map_err(|e| Error::new(ErrorKind::Other, e))
    }
}
