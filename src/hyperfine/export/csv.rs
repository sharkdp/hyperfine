use super::{ExportEntry, ResultExporter};

use csv::WriterBuilder;
use std::io::Result;

pub struct CsvExporter {
    out_file: String,
}

impl ResultExporter for CsvExporter {
    fn write(&self, results: &Vec<ExportEntry>) -> Result<()> {
        let mut writer = WriterBuilder::new().from_path(&self.out_file)?;
        for res in results {
            writer.serialize(res)?;
        }
        Ok(())
    }
}

impl CsvExporter {
    pub fn new(file_name: String) -> Self {
        CsvExporter {
            out_file: file_name,
        }
    }
}
