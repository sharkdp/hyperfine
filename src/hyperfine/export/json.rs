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
        let serialized = to_string_pretty(&HyperfineSummary { results });

        serialized.map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Error while serializing to JSON: {:}", e),
            )
        })
    }
}

impl JsonExporter {
    pub fn new() -> Self {
        JsonExporter {}
    }
}
