use std::fs::File;
use super::Exporter;

use crate::hyperfine::types::BenchmarkResult;
use crate::hyperfine::units::Unit;

use std::io::{Error, ErrorKind, Result};

use csv::WriterBuilder;
use csv::Writer;

pub struct CsvExporter {
    writer: Writer<File>
}
impl CsvExporter { 
    pub fn new(filename: &str) -> Self { 
        CsvExporter { 
            writer: WriterBuilder::new().from_path(filename).unwrap()
        }
    }
}
impl Exporter for CsvExporter {
    fn serialize(&self, results: &[BenchmarkResult], _unit: Option<Unit>) -> Result<Vec<u8>> {
        let mut writer = WriterBuilder::new().from_writer(vec![]);
        for result in results {
            // The list of times cannot be exported to the CSV file - remove it:
            let mut result = result.clone();
            result.times = None;
            writer.serialize(result)?;
        }
        
        writer
            .into_inner()
            .map_err(|e| Error::new(ErrorKind::Other, e))
    }
    fn write_to_file_incremental(&mut self, result: &BenchmarkResult, _unit: Option<Unit>) -> Result<()> { 
        strip_times_and_write(&mut self.writer, result)?;
        self.writer.flush()
    }
    fn supports_incremental_writes(&self) -> bool { 
        true
    }
}

fn strip_times_and_write(writer: &mut Writer<File>, result: &BenchmarkResult) -> Result<()> { 
    // The list of times cannot be exported to the CSV file - remove it:
    let mut result = result.clone();
    result.times = None;
    writer.serialize(result)
        .map_err(|e| Error::new(ErrorKind::Other, e))
}

/// Integration test
#[test]
fn test_lines_are_appended() { 
    use std::io::Read;

    let path = &String::from("incremental.csv");
    let mut exporter = CsvExporter::new(path);
    
    let results = vec![
        BenchmarkResult::new(
            String::from("command | 1"),
            1.0,
            2.0,
            1.0,
            3.0,
            4.0,
            5.0,
            6.0,
            vec![7.0, 8.0, 9.0],
            None,
        ),
        BenchmarkResult::new(
            String::from("command | 2"),
            11.0,
            12.0,
            11.0,
            13.0,
            14.0,
            15.0,
            16.0,
            vec![17.0, 18.0, 19.0],
            None,
        ),
    ];

    for mut result in results {
        exporter.write_to_file_incremental(&mut result, Some(Unit::Second)).unwrap();
        let metadata = std::fs::metadata(path).unwrap();
        println!("{}b", metadata.len())
    }

    let mut buffer = String::new();
    std::fs::File::open(path).unwrap()
        .read_to_string(&mut buffer).unwrap();

    let expected = String::from("\
    command,mean,stddev,median,user,system,min,max\n\
    command | 1,1.0,2.0,1.0,3.0,4.0,5.0,6.0\n\
    command | 2,11.0,12.0,11.0,13.0,14.0,15.0,16.0\n");

    assert_eq!(expected, buffer);
}