use super::{ExportEntry, Exporter};

use std::io::{Error, ErrorKind, Result};

use serde_json::to_string_pretty;

#[derive(Serialize, Debug)]
struct HyperfineSummary<'a> {
    results: &'a Vec<ExportEntry>,
}

pub struct JsonExporter {}

impl Exporter for JsonExporter {
    fn serialize(&self, results: &Vec<ExportEntry>) -> Result<String> {
        to_string_pretty(&HyperfineSummary { results }).map_err(|e| Error::new(ErrorKind::Other, e))
    }
}

impl JsonExporter {
    pub fn new() -> Self {
        JsonExporter {}
    }
}
