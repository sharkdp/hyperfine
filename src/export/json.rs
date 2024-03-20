use serde::*;
use serde_json::to_vec_pretty;

use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::options::SortOrder;
use crate::util::units::Unit;

use anyhow::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct HyperfineSummary {
    pub results: Vec<BenchmarkResult>,
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
        let mut output = to_vec_pretty(&HyperfineSummary {
            results: results.to_vec(),
        });
        if let Ok(ref mut content) = output {
            content.push(b'\n');
        }

        Ok(output?)
    }
}
