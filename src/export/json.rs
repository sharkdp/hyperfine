use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::to_vec_pretty;

use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::options::SortOrder;
use crate::util::units::Unit;

use anyhow::{Context, Result};

#[derive(Serialize, Deserialize, Debug)]
struct HyperfineSummary {
    results: Vec<BenchmarkResult>,
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

pub fn load_benchmark_results(path: &Path) -> Result<Vec<BenchmarkResult>> {
    let content = fs::read(path)
        .with_context(|| format!("Could not read JSON export file '{}'", path.display()))?;
    let mut summary: HyperfineSummary = serde_json::from_slice(&content)
        .with_context(|| format!("Could not parse JSON export file '{}'", path.display()))?;

    for result in &mut summary.results {
        if result.command_with_unused_parameters.is_empty() {
            result.command_with_unused_parameters = result.command.clone();
        }
    }

    Ok(summary.results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    use tempfile::NamedTempFile;

    #[test]
    fn load_benchmark_results_restores_display_command() {
        let exporter = JsonExporter::default();
        let results = vec![BenchmarkResult {
            command: "sleep 0.1".into(),
            command_with_unused_parameters: "sleep 0.1".into(),
            mean: 0.1,
            stddev: None,
            median: 0.1,
            user: 0.0,
            system: 0.0,
            min: 0.1,
            max: 0.1,
            times: Some(vec![0.1]),
            memory_usage_byte: None,
            exit_codes: vec![Some(0)],
            parameters: Default::default(),
        }];

        let json = exporter
            .serialize(&results, None, SortOrder::Command)
            .unwrap();
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&json).unwrap();

        let loaded = load_benchmark_results(file.path()).unwrap();
        assert_eq!(loaded, results);
    }
}
