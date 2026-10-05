use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::benchmark::benchmark_result::BenchmarkResult;

/// Mirror of the structure written by `JsonExporter` so we can round-trip
/// previously-saved benchmark runs back into hyperfine.
#[derive(Deserialize)]
struct ImportedSummary {
    results: Vec<BenchmarkResult>,
}

/// Read a previously-saved JSON file and return the benchmark results inside it.
///
/// The file is expected to follow the same schema written by `--export-json`,
/// i.e. a top-level object with a `"results"` array of benchmark objects.
pub fn load_results_from_json<P: AsRef<Path>>(path: P) -> Result<Vec<BenchmarkResult>> {
    let path = path.as_ref();
    let file = File::open(path)
        .with_context(|| format!("Could not open import file '{}'", path.display()))?;
    let reader = BufReader::new(file);
    let summary: ImportedSummary = serde_json::from_reader(reader)
        .with_context(|| format!("Failed to parse JSON from import file '{}'", path.display()))?;

    let mut results = summary.results;
    for result in &mut results {
        // The JSON schema does not carry `command_with_unused_parameters`, since
        // `--export-json` skips it. Fall back to the command line itself so that
        // the imported entry still has something sensible to display.
        if result.command_with_unused_parameters.is_empty() {
            result.command_with_unused_parameters = result.command.clone();
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::load_results_from_json;
    use crate::benchmark::benchmark_result::BenchmarkResult;

    fn write_temp_json(contents: &str) -> std::path::PathBuf {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("results.json");
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
        // Leak the tempdir so the file survives for the duration of the test.
        let _ = dir.keep();
        path
    }

    #[test]
    fn round_trip_export_then_import() {
        use serde_json::json;

        let original = vec![
            BenchmarkResult {
                command: "sleep 0.1".into(),
                command_with_unused_parameters: "sleep 0.1".into(),
                mean: 0.1,
                stddev: Some(0.01),
                median: 0.1,
                user: 0.05,
                system: 0.02,
                min: 0.09,
                max: 0.11,
                times: Some(vec![0.1, 0.1]),
                memory_usage_byte: None,
                exit_codes: vec![Some(0), Some(0)],
                parameters: BTreeMap::new(),
            },
            BenchmarkResult {
                command: "sleep 0.2".into(),
                command_with_unused_parameters: "sleep 0.2".into(),
                mean: 0.2,
                stddev: Some(0.02),
                median: 0.2,
                user: 0.1,
                system: 0.05,
                min: 0.18,
                max: 0.22,
                times: Some(vec![0.2, 0.2]),
                memory_usage_byte: None,
                exit_codes: vec![Some(0), Some(0)],
                parameters: BTreeMap::new(),
            },
        ];

        // Re-emit via serde to confirm the schema written by `--export-json`
        // round-trips back to a `BenchmarkResult` without information loss.
        let summary = json!({ "results": &original });
        let path = write_temp_json(&summary.to_string());

        let imported = load_results_from_json(&path).unwrap();
        assert_eq!(imported.len(), original.len());
        for (a, b) in imported.iter().zip(original.iter()) {
            assert_eq!(a.command, b.command);
            assert_eq!(a.mean, b.mean);
            assert_eq!(a.stddev, b.stddev);
            assert_eq!(a.median, b.median);
            assert_eq!(a.times, b.times);
            assert_eq!(a.exit_codes, b.exit_codes);
        }
    }

    #[test]
    fn missing_optional_fields_are_tolerated() {
        let path = write_temp_json(
            r#"{
                "results": [
                    {
                        "command": "echo a",
                        "mean": 0.5,
                        "stddev": null,
                        "median": 0.5,
                        "user": 0.0,
                        "system": 0.0,
                        "min": 0.5,
                        "max": 0.5,
                        "exit_codes": [0]
                    }
                ]
            }"#,
        );
        let results = load_results_from_json(&path).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].command, "echo a");
        assert_eq!(results[0].command_with_unused_parameters, "echo a");
        assert_eq!(results[0].mean, 0.5);
        assert!(results[0].times.is_none());
    }

    #[test]
    fn missing_file_produces_a_helpful_error() {
        let err = load_results_from_json("/no/such/file/hopefully.json").unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("Could not open import file"));
    }
}
