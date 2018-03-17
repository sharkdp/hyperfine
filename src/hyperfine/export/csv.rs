use super::{ExportEntry, ResultExporter};

use csv::WriterBuilder;
use std::io::Result;

pub struct CsvExporter {
    results: Vec<ExportEntry>,
    out_file: String,
}

impl ResultExporter for CsvExporter {
    fn add_entry(&mut self, entry: ExportEntry) {
        self.results.push(entry);
    }

    fn write(&self) -> Result<()> {
        let mut writer = WriterBuilder::new().from_path(&self.out_file)?;
        for res in &self.results {
            writer.serialize(res)?;
        }
        Ok(())
    }
}

impl CsvExporter {
    pub fn new(file_name: String) -> Self {
        CsvExporter {
            results: Vec::new(),
            out_file: file_name,
        }
    }
}
