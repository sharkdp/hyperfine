use serde::*;
use serde_json::to_vec_pretty;

use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::options::SortOrder;
use crate::quantity::TimeUnit;

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
        _time_unit: Option<TimeUnit>,
        _sort_order: SortOrder,
        _reference_index: Option<usize>,
    ) -> Result<Vec<u8>> {
        let mut output = to_vec_pretty(&HyperfineSummary { results });
        if let Ok(ref mut content) = output {
            content.push(b'\n');
        }

        Ok(output?)
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
        let output = JsonExporter::default()
            .serialize(&[result], None, SortOrder::Command)
            .unwrap();
        let actual: serde_json::Value = serde_json::from_slice(&output).unwrap();
        let mut expected = json!({
            "results": [{
                "command": "example",
                "measurements": [{
                    "time_wall_clock": {"value": 0.0, "unit": "second"},
                    "time_user": {"value": 0.0, "unit": "second"},
                    "time_system": {"value": 0.0, "unit": "second"},
                    "exit_code": 0
                }]
            }]
        });
        if let Some(memory) = expected_memory {
            expected["results"][0]["measurements"][0]["peak_memory_usage"] = memory;
        }
        assert_eq!(actual, expected);
    }
}
