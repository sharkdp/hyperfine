use super::{ExportEntry, ResultExporter};

use std::io::Result;
use std::fs::File;

// TODO: Investigate output to non-pretty formats
use serde_json::to_writer_pretty;

#[derive(Serialize, Debug)]
struct HyperfineSummary<'a> {
    results: &'a Vec<ExportEntry>,
}

pub struct JsonExporter {
    out_file: String,
}

impl ResultExporter for JsonExporter {
    fn write(&self, results: &Vec<ExportEntry>) -> Result<()> {
        let file: File = File::create(&self.out_file)?;
        to_writer_pretty(file, &HyperfineSummary { results })?;
        Ok(())
    }
}

impl JsonExporter {
    pub fn new(file_name: String) -> Self {
        JsonExporter {
            out_file: file_name,
        }
    }
}
