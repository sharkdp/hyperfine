use super::Exporter;
use crate::hyperfine::types::BenchmarkResult;
use crate::hyperfine::units::Unit;

use std::io::{Error, ErrorKind, Result};

use serde::*;
use serde_json::to_vec_pretty;

#[derive(Serialize, Debug)]
struct HyperfineSummary<'a> {
    results: &'a [BenchmarkResult],
}

#[derive(Default)]
pub struct JsonExporter {}

impl Exporter for JsonExporter {
    fn serialize(&self, results: &[BenchmarkResult], _unit: Option<Unit>) -> Result<Vec<u8>> {
        let mut output = to_vec_pretty(&HyperfineSummary { results });
        for content in output.iter_mut() {
            content.push(b'\n');
        }

        output.map_err(|e| Error::new(ErrorKind::Other, e))
    }
}
