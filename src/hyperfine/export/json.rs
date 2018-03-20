use super::{ExportEntry, Exporter};

use std::io::{Error, ErrorKind, Result};

use serde_json::to_vec_pretty;

#[derive(Serialize, Debug)]
struct HyperfineSummary<'a> {
    results: &'a Vec<ExportEntry>,
}

pub struct JsonExporter {}

impl Exporter for JsonExporter {
    fn serialize(&self, results: &Vec<ExportEntry>) -> Result<Vec<u8>> {
        let serialized = to_vec_pretty(&HyperfineSummary { results });

        match serialized {
            Ok(file_content) => Ok(file_content),
            Err(e) => Err(Error::new(ErrorKind::Other, format!("{:?}", e))),
        }
    }
}

impl JsonExporter {
    pub fn new() -> Self {
        JsonExporter {}
    }
}
