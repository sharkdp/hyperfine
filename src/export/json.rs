use std::ops::{Add, AddAssign, Div};

use serde::*;
use serde_json::to_vec_pretty;

use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::benchmark::measurement::Measurements;
use crate::options::SortOrder;
use crate::quantity::statistics::{max, mean, median, min, standard_deviation, UnsafeRawValue};
use crate::quantity::{byte, ratio, second, QuantityInUnit, Ratio, TimeUnit, Zero};

use anyhow::Result;

mod metadata;
use metadata::Metadata;

const JSON_SCHEMA_VERSION: u32 = 2;

#[derive(Serialize)]
struct HyperfineSummary<'a> {
    schema_version: u32,
    metadata: &'a Metadata,
    results: Vec<JsonBenchmarkResult<'a>>,
}

#[derive(Serialize)]
struct JsonBenchmarkResult<'a> {
    #[serde(flatten)]
    result: &'a BenchmarkResult,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    is_reference: bool,
    summary: BenchmarkSummary,
}

#[derive(Serialize)]
struct BenchmarkSummary {
    time_wall_clock: StatisticalSummary,
    time_user: StatisticalSummary,
    time_system: StatisticalSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory_peak_resident: Option<StatisticalSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu_cycles: Option<StatisticalSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<StatisticalSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_references: Option<StatisticalSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_misses: Option<StatisticalSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    branch_misses: Option<StatisticalSummary>,
}

impl BenchmarkSummary {
    fn from_measurements(measurements: &Measurements) -> Self {
        let values = &measurements.measurements;
        Self {
            time_wall_clock: StatisticalSummary::from_values(
                values.iter().map(|m| Some(m.time_wall_clock)),
                Some(second),
            )
            .expect("Benchmarks have at least one measurement"),
            time_user: StatisticalSummary::from_values(
                values.iter().map(|m| Some(m.time_user)),
                Some(second),
            )
            .expect("Benchmarks have at least one measurement"),
            time_system: StatisticalSummary::from_values(
                values.iter().map(|m| Some(m.time_system)),
                Some(second),
            )
            .expect("Benchmarks have at least one measurement"),
            memory_peak_resident: StatisticalSummary::from_values(
                values
                    .iter()
                    .filter_map(|m| m.memory_peak_resident)
                    .map(Some),
                Some(byte),
            ),
            cpu_cycles: StatisticalSummary::from_values(
                values.iter().map(|m| {
                    m.hardware_counters
                        .cpu_cycles
                        .map(|v| Ratio::new::<ratio>(v as f64))
                }),
                None::<ratio>,
            ),
            instructions: StatisticalSummary::from_values(
                values.iter().map(|m| {
                    m.hardware_counters
                        .instructions
                        .map(|v| Ratio::new::<ratio>(v as f64))
                }),
                None::<ratio>,
            ),
            cache_references: StatisticalSummary::from_values(
                values.iter().map(|m| {
                    m.hardware_counters
                        .cache_references
                        .map(|v| Ratio::new::<ratio>(v as f64))
                }),
                None::<ratio>,
            ),
            cache_misses: StatisticalSummary::from_values(
                values.iter().map(|m| {
                    m.hardware_counters
                        .cache_misses
                        .map(|v| Ratio::new::<ratio>(v as f64))
                }),
                None::<ratio>,
            ),
            branch_misses: StatisticalSummary::from_values(
                values.iter().map(|m| {
                    m.hardware_counters
                        .branch_misses
                        .map(|v| Ratio::new::<ratio>(v as f64))
                }),
                None::<ratio>,
            ),
        }
    }
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
    fn from_values<Q, P, U>(
        values: impl IntoIterator<Item = Option<Q>>,
        unit: Option<U>,
    ) -> Option<Self>
    where
        Q: Copy
            + PartialOrd
            + Add<Output = Q>
            + AddAssign
            + Zero
            + Div<Ratio, Output = P>
            + UnsafeRawValue
            + QuantityInUnit<U>,
        P: Into<Q>,
        U: uom::si::Unit,
    {
        // Only summarize a metric when it is available for every supplied measurement.
        let values: Vec<_> = values.into_iter().collect::<Option<_>>()?;
        if values.is_empty() {
            return None;
        }
        Some(Self {
            unit: unit.map(|_| U::singular()),
            count: values.len(),
            mean: mean(values.iter().copied()).value_in_unit(),
            stddev: (values.len() > 1)
                .then(|| standard_deviation(values.iter().copied()).value_in_unit()),
            median: median(values.iter().copied()).value_in_unit(),
            min: min(values.iter().copied()).value_in_unit(),
            max: max(values.iter().copied()).value_in_unit(),
        })
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
        _time_unit: Option<TimeUnit>,
        _sort_order: SortOrder,
        reference_index: Option<usize>,
    ) -> Result<Vec<u8>> {
        let mut output = to_vec_pretty(&HyperfineSummary {
            schema_version: JSON_SCHEMA_VERSION,
            metadata: &self.metadata,
            results: results
                .iter()
                .enumerate()
                .map(|(index, result)| JsonBenchmarkResult {
                    result,
                    is_reference: reference_index == Some(index),
                    summary: BenchmarkSummary::from_measurements(&result.measurements),
                })
                .collect(),
        });
        if let Ok(ref mut content) = output {
            content.push(b'\n');
        }

        Ok(output?)
    }
}

#[test]
fn test_statistical_summary_unit_conversion() {
    use crate::quantity::{kibibyte, millisecond, Information, Time};

    let time = StatisticalSummary::from_values(
        [Time::new::<second>(1.0), Time::new::<second>(3.0)].map(Some),
        Some(millisecond),
    )
    .unwrap();
    let information = StatisticalSummary::from_values(
        [
            Information::new::<byte>(1024.0),
            Information::new::<byte>(3072.0),
        ]
        .map(Some),
        Some(kibibyte),
    )
    .unwrap();

    for (summary, unit, scale) in [
        (time, "millisecond", 1000.0),
        (information, "kibibyte", 1.0),
    ] {
        assert_eq!(summary.unit, Some(unit));
        assert_eq!(summary.count, 2);
        approx::assert_relative_eq!(summary.mean, 2.0 * scale);
        approx::assert_relative_eq!(summary.median, 2.0 * scale);
        approx::assert_relative_eq!(summary.min, scale);
        approx::assert_relative_eq!(summary.max, 3.0 * scale);
        approx::assert_relative_eq!(summary.stddev.unwrap(), 2.0_f64.sqrt() * scale);
    }
}
