use serde::*;
use serde_json::to_vec_pretty;

use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::options::SortOrder;
use crate::util::units::Unit;

use anyhow::Result;

#[derive(Serialize, Debug)]
struct HyperfineSummary<'a> {
    results: &'a [BenchmarkResult],
}

#[derive(Default)]
pub struct JsonExporter {}

impl Exporter for JsonExporter {
    fn serialize(
        &self,
        results: &[BenchmarkResult],
        _unit: Option<Unit>,
        _sort_order: SortOrder,
    ) -> Result<Vec<u8>> {
        let mut output = to_vec_pretty(&HyperfineSummary { results });
        if let Ok(ref mut content) = output {
            content.push(b'\n');
        }

        Ok(output?)
    }
}
