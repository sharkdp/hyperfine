use super::{ExportEntry, ResultExporter};

use std::io::{Error, ErrorKind, Result};

// TODO: Investigate output to non-pretty formats
use serde_json::to_vec_pretty;

#[derive(Serialize, Debug)]
struct HyperfineSummary<'a> {
    results: &'a Vec<ExportEntry>,
}

pub struct JsonExporter {}

impl ResultExporter for JsonExporter {
    fn write(&self, results: &Vec<ExportEntry>) -> Result<Vec<u8>> {
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
