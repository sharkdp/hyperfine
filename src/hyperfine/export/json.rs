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
        let mut output = to_vec_pretty(&HyperfineSummary { results });
        for content in output.iter_mut() {
            content.push(b'\n');
        }

        output.map_err(|e| Error::new(ErrorKind::Other, e))
    }
}
