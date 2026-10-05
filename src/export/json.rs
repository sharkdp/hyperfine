use std::ops::{Add, AddAssign, Div};

use serde::*;
use serde_json::to_vec_pretty;

use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::benchmark::measurement::Measurements;
use crate::options::SortOrder;
use crate::quantity::statistics::{max, mean, median, min, standard_deviation, UnsafeRawValue};
use crate::quantity::{byte, second, QuantityInUnit, Ratio, TimeUnit, Zero};

use anyhow::Result;

mod metadata;
use metadata::Metadata;

#[derive(Serialize)]
struct HyperfineSummary<'a> {
    metadata: &'a Metadata,
    results: Vec<JsonBenchmarkResult<'a>>,
}

#[derive(Serialize)]
struct JsonBenchmarkResult<'a> {
    #[serde(flatten)]
    result: &'a BenchmarkResult,
    summary: BenchmarkSummary,
}

#[derive(Serialize)]
struct BenchmarkSummary {
    time_wall_clock: StatisticalSummary,
    time_user: StatisticalSummary,
    time_system: StatisticalSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    peak_memory_usage: Option<StatisticalSummary>,
}

impl BenchmarkSummary {
    fn from_measurements(measurements: &Measurements) -> Self {
        let values = &measurements.measurements;
        let mut memory_values = values.iter().filter_map(|m| m.peak_memory_usage).peekable();
        Self {
            time_wall_clock: StatisticalSummary::from_values(
                values.iter().map(|m| m.time_wall_clock),
                second,
            ),
            time_user: StatisticalSummary::from_values(values.iter().map(|m| m.time_user), second),
            time_system: StatisticalSummary::from_values(
                values.iter().map(|m| m.time_system),
                second,
            ),
            peak_memory_usage: memory_values
                .peek()
                .is_some()
                .then(|| StatisticalSummary::from_values(memory_values, byte)),
        }
    }
}

#[derive(Serialize)]
struct StatisticalSummary {
    unit: &'static str,
    count: usize,
    mean: f64,
    stddev: Option<f64>,
    median: f64,
    min: f64,
    max: f64,
}

impl StatisticalSummary {
    fn from_values<Q, P, U>(values: impl IntoIterator<Item = Q>, _unit: U) -> Self
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
        let values: Vec<_> = values.into_iter().collect();
        assert!(
            !values.is_empty(),
            "Statistical summaries require at least one measurement"
        );
        Self {
            unit: U::singular(),
            count: values.len(),
            mean: mean(values.iter().copied()).value_in_unit(),
            stddev: (values.len() > 1)
                .then(|| standard_deviation(values.iter().copied()).value_in_unit()),
            median: median(values.iter().copied()).value_in_unit(),
            min: min(values.iter().copied()).value_in_unit(),
            max: max(values.iter().copied()).value_in_unit(),
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
        _time_unit: Option<TimeUnit>,
        _sort_order: SortOrder,
        _reference_index: Option<usize>,
    ) -> Result<Vec<u8>> {
        let mut output = to_vec_pretty(&HyperfineSummary {
            metadata: &self.metadata,
            results: results
                .iter()
                .map(|result| JsonBenchmarkResult {
                    result,
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
        [Time::new::<second>(1.0), Time::new::<second>(3.0)],
        millisecond,
    );
    let information = StatisticalSummary::from_values(
        [
            Information::new::<byte>(1024.0),
            Information::new::<byte>(3072.0),
        ],
        kibibyte,
    );

    for (summary, unit, scale) in [
        (time, "millisecond", 1000.0),
        (information, "kibibyte", 1.0),
    ] {
        assert_eq!(summary.unit, unit);
        assert_eq!(summary.count, 2);
        approx::assert_relative_eq!(summary.mean, 2.0 * scale);
        approx::assert_relative_eq!(summary.median, 2.0 * scale);
        approx::assert_relative_eq!(summary.min, scale);
        approx::assert_relative_eq!(summary.max, 3.0 * scale);
        approx::assert_relative_eq!(summary.stddev.unwrap(), 2.0_f64.sqrt() * scale);
    }
}

#[test]
fn test_json_optional_memory() {
    use crate::benchmark::measurement::{Measurement, Measurements};
    use crate::quantity::{byte, Information};
    use serde_json::json;

    for (memory, expected_memory) in [
        (None, None),
        (
            Some(Information::new::<byte>(0.0)),
            Some(json!({"value": 0.0, "unit": "byte"})),
        ),
        (
            Some(Information::new::<byte>(1024.0)),
            Some(json!({"value": 1024.0, "unit": "byte"})),
        ),
    ] {
        let result = BenchmarkResult {
            command: "example".into(),
            measurements: Measurements::new(vec![Measurement {
                peak_memory_usage: memory,
                ..Measurement::default()
            }]),
            ..BenchmarkResult::default()
        };
        let mut exporter = JsonExporter::default();
        exporter.metadata.start_time = Some("2000-02-29T12:34:56Z".to_owned());
        let output = exporter
            .serialize(&[result], None, SortOrder::Command, None)
            .unwrap();
        let actual: serde_json::Value = serde_json::from_slice(&output).unwrap();
        let mut expected = json!({
            "metadata": {
                "json_schema_version": metadata::JSON_SCHEMA_VERSION,
                "hyperfine_version": env!("CARGO_PKG_VERSION"),
                "start_time": "2000-02-29T12:34:56Z",
                "platform": exporter.metadata.platform
            },
            "results": [{
                "command": "example",
                "summary": {
                    "time_wall_clock": {
                        "unit": "second",
                        "count": 1,
                        "mean": 0.0,
                        "stddev": null,
                        "median": 0.0,
                        "min": 0.0,
                        "max": 0.0
                    }
                },
                "measurements": [{
                    "time_wall_clock": {"value": 0.0, "unit": "second"},
                    "time_user": {"value": 0.0, "unit": "second"},
                    "time_system": {"value": 0.0, "unit": "second"},
                    "exit_code": 0
                }]
            }]
        });
        for field in ["time_user", "time_system"] {
            expected["results"][0]["summary"][field] =
                expected["results"][0]["summary"]["time_wall_clock"].clone();
        }
        if let Some(memory) = expected_memory {
            let value = memory["value"].clone();
            expected["results"][0]["summary"]["peak_memory_usage"] = json!({
                "unit": "byte", "count": 1, "mean": value,
                "stddev": null, "median": value, "min": value, "max": value
            });
            expected["results"][0]["measurements"][0]["peak_memory_usage"] = memory;
        }
        assert_eq!(actual, expected);
    }
}

#[test]
fn test_json_summaries_for_parameterized_results() {
    use crate::benchmark::benchmark_result::Parameter;
    use crate::benchmark::measurement::Measurement;
    use crate::quantity::{second, Time};
    use serde_json::json;

    let results: Vec<_> = vec![("small", vec![1.0, 3.0]), ("large", vec![10.0])]
        .into_iter()
        .map(|(size, times)| BenchmarkResult {
            command: "example".into(),
            parameters: [(
                "size".into(),
                Parameter {
                    value: size.into(),
                    is_unused: true,
                },
            )]
            .into(),
            measurements: Measurements::new(
                times
                    .into_iter()
                    .map(|time| Measurement {
                        time_wall_clock: Time::new::<second>(time),
                        ..Measurement::default()
                    })
                    .collect(),
            ),
        })
        .collect();
    let output = JsonExporter::default()
        .serialize(&results, None, SortOrder::Command, None)
        .unwrap();
    let actual: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let results = actual["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0]["parameters"]["size"]["value"], "small");
    assert_eq!(results[1]["parameters"]["size"]["value"], "large");
    assert_eq!(results[0]["measurements"].as_array().unwrap().len(), 2);
    assert_eq!(results[1]["measurements"].as_array().unwrap().len(), 1);
    assert_eq!(
        results[0]["summary"]["time_wall_clock"],
        json!({
            "unit": "second", "count": 2, "mean": 2.0,
            "stddev": 2.0_f64.sqrt(), "median": 2.0, "min": 1.0, "max": 3.0
        })
    );
    assert_eq!(
        results[1]["summary"]["time_wall_clock"],
        json!({
            "unit": "second", "count": 1, "mean": 10.0,
            "stddev": null, "median": 10.0, "min": 10.0, "max": 10.0
        })
    );
}

#[test]
fn test_json_summary_metrics_and_missing_samples() {
    use crate::benchmark::measurement::Measurement;
    use crate::quantity::{byte, second, Information, Time};
    use serde_json::json;

    let measurements = Measurements::new(
        vec![(1.0, Some(1024.0)), (2.0, None), (3.0, Some(3072.0))]
            .into_iter()
            .map(|(time, memory)| Measurement {
                time_wall_clock: Time::new::<second>(time),
                time_user: Time::new::<second>(time * 2.0),
                time_system: Time::new::<second>(time * 3.0),
                peak_memory_usage: memory.map(Information::new::<byte>),
                ..Measurement::default()
            })
            .collect(),
    );
    let actual = serde_json::to_value(BenchmarkSummary::from_measurements(&measurements)).unwrap();
    assert_eq!(
        actual,
        json!({
            "time_wall_clock": {
                "unit": "second", "count": 3, "mean": 2.0, "stddev": 1.0,
                "median": 2.0, "min": 1.0, "max": 3.0
            },
            "time_user": {
                "unit": "second", "count": 3, "mean": 4.0, "stddev": 2.0,
                "median": 4.0, "min": 2.0, "max": 6.0
            },
            "time_system": {
                "unit": "second", "count": 3, "mean": 6.0, "stddev": 3.0,
                "median": 6.0, "min": 3.0, "max": 9.0
            },
            "peak_memory_usage": {
                "unit": "byte", "count": 2, "mean": 2048.0,
                "stddev": 1024.0 * 2.0_f64.sqrt(),
                "median": 2048.0, "min": 1024.0, "max": 3072.0
            }
        })
    );
}
