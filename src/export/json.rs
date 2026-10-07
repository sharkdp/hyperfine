use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::to_vec_pretty;

use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::metric::{Metric, MetricSelection, Stats};

use anyhow::Result;

mod metadata;
use metadata::Metadata;

const JSON_SCHEMA_VERSION: u32 = 2;

#[derive(Serialize)]
struct HyperfineSummary<'a> {
    schema_version: u32,
    metadata: &'a Metadata,
    primary_metric: &'static str,
    results: Vec<JsonBenchmarkResult<'a>>,
}

#[derive(Serialize)]
struct JsonBenchmarkResult<'a> {
    #[serde(flatten)]
    result: &'a BenchmarkResult,
    summary: BTreeMap<&'static str, StatisticalSummary>,
}

#[derive(Serialize)]
struct StatisticalSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    unit: Option<&'static str>,
    count: usize,
    mean: f64,
    stddev: Option<f64>,
    median: f64,
    min: f64,
    max: f64,
}

impl StatisticalSummary {
    fn new(metric: Metric, stats: Stats) -> Self {
        Self {
            unit: match metric.base_unit().symbol {
                "s" => Some("second"),
                "B" => Some("byte"),
                _ => None,
            },
            count: stats.count,
            mean: stats.mean,
            stddev: stats.stddev,
            median: stats.median,
            min: stats.min,
            max: stats.max,
        }
    }
}

#[derive(Default)]
pub struct JsonExporter {
    metadata: Metadata,
}

impl Exporter for JsonExporter {
    fn serialize(
        &self,
        results: &[BenchmarkResult],
        metrics: &[MetricSelection],
    ) -> Result<Vec<u8>> {
        let primary = metrics[0];
        let mut output = to_vec_pretty(&HyperfineSummary {
            schema_version: JSON_SCHEMA_VERSION,
            metadata: &self.metadata,
            primary_metric: primary.metric.name(),
            results: results
                .iter()
                .map(|result| JsonBenchmarkResult {
                    result,
                    summary: Metric::ALL
                        .iter()
                        .copied()
                        .filter_map(|metric| {
                            metric.summarize(&result.measurements).map(|stats| {
                                (metric.name(), StatisticalSummary::new(metric, stats))
                            })
                        })
                        .collect(),
                })
                .collect(),
        })?;
        output.push(b'\n');
        Ok(output)
    }
}
