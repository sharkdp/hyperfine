use super::Exporter;
use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::benchmark::relative_speed;
use crate::options::SortOrder;
use crate::util::units::Unit;

use anyhow::Result;
use serde_json;

/// HTML exporter for benchmark results.
///
/// Generates a standalone HTML file with interactive visualizations using Plotly.js
#[derive(Default)]
pub struct HtmlExporter {}

impl Exporter for HtmlExporter {
    fn serialize(
        &self,
        results: &[BenchmarkResult],
        unit: Option<Unit>,
        sort_order: SortOrder,
    ) -> Result<Vec<u8>> {
        // Include static assets
        let template = include_str!("html_template.html");
        let css = include_str!("html_styles.css");
        let js = include_str!("html_renderer.js");

        // Build the HTML document with embedded resources
        let mut html = template.to_string();
        html = html.replace("/* CSS will be embedded here */", css);
        html = html.replace("// JS will be embedded here", js);

        // Determine the appropriate unit if not specified
        let unit = unit.unwrap_or_else(|| determine_unit_from_results(results));

        // Compute relative speeds and sort results
        let entries = relative_speed::compute(results, sort_order);

        // Get the reference command if there is one
        let reference_command = entries
            .iter()
            .find(|e| e.is_reference)
            .map_or("", |e| &e.result.command);

        // Serialize benchmark data to JSON for JavaScript consumption
        let json_data = serde_json::to_string(
            &entries
                .iter()
                .map(|entry| &entry.result)
                .collect::<Vec<_>>(),
        )?;

        // Replace placeholder with benchmark data and unit information
        let data_script = format!(
            "const benchmarkData = {};\n\
             const unitShortName = \"{}\";\n\
             const unitName = \"{}\";\n\
             const referenceCommand = \"{}\";\n\
             const unitFactor = {};",
            json_data,
            get_unit_short_name(unit),
            get_unit_name(unit),
            reference_command,
            get_unit_factor(unit)
        );

        html = html.replace("<!-- DATA_PLACEHOLDER -->", &data_script);

        Ok(html.into_bytes())
    }
}

/// Returns the full name of a time unit
fn get_unit_name(unit: Unit) -> &'static str {
    match unit {
        Unit::Second => "second",
        Unit::MilliSecond => "millisecond",
        Unit::MicroSecond => "microsecond",
    }
}

/// Returns the abbreviated symbol for a time unit
fn get_unit_short_name(unit: Unit) -> &'static str {
    match unit {
        Unit::Second => "s",
        Unit::MilliSecond => "ms",
        Unit::MicroSecond => "μs",
    }
}

/// Returns the conversion factor from seconds to the specified unit
fn get_unit_factor(unit: Unit) -> f64 {
    match unit {
        Unit::Second => 1.0,
        Unit::MilliSecond => 1000.0,
        Unit::MicroSecond => 1000000.0,
    }
}

/// Automatically determines the most appropriate time unit based on benchmark results
fn determine_unit_from_results(results: &[BenchmarkResult]) -> Unit {
    results
        .first()
        .map(|first_result| {
            // Choose unit based on the magnitude of the mean time
            let mean = first_result.mean;
            if mean < 0.001 {
                Unit::MicroSecond
            } else if mean < 1.0 {
                Unit::MilliSecond
            } else {
                Unit::Second
            }
        })
        .unwrap_or(Unit::Second) // Default to seconds if no results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_html_export() {
        // Create sample benchmark results
        let results = vec![
            create_test_benchmark("test command 1", 1.5, None),
            create_test_benchmark_with_param("test command 2", 2.5, "size", "10"),
        ];

        // Create HTML exporter
        let exporter = HtmlExporter::default();

        // Test with seconds unit
        let html = export_and_get_html(&exporter, &results, Unit::Second);

        // Verify HTML structure and content
        assert_html_structure(&html);
        assert_contains_benchmark_data(&html, &results);
        assert_unit_information(&html, "s", "second", "1");

        // Test with milliseconds unit
        let html = export_and_get_html(&exporter, &results, Unit::MilliSecond);
        assert_unit_information(&html, "ms", "millisecond", "1000");
    }

    /// Helper function to create a test benchmark result
    fn create_test_benchmark(
        command: &str,
        mean: f64,
        parameters: Option<BTreeMap<String, String>>,
    ) -> BenchmarkResult {
        BenchmarkResult {
            command: command.to_string(),
            command_with_unused_parameters: command.to_string(),
            mean,
            stddev: Some(mean * 0.1),
            median: mean * 0.99,
            min: mean * 0.8,
            max: mean * 1.2,
            user: mean * 0.9,
            system: mean * 0.1,
            memory_usage_byte: None,
            times: Some(vec![mean * 0.8, mean * 0.9, mean, mean * 1.1, mean * 1.2]),
            exit_codes: vec![Some(0); 5],
            parameters: parameters.unwrap_or_default(),
        }
    }

    /// Helper function to create a test benchmark with a parameter
    fn create_test_benchmark_with_param(
        command: &str,
        mean: f64,
        param_name: &str,
        param_value: &str,
    ) -> BenchmarkResult {
        let mut params = BTreeMap::new();
        params.insert(param_name.to_string(), param_value.to_string());
        create_test_benchmark(command, mean, Some(params))
    }

    /// Helper function to export benchmark results and get HTML
    fn export_and_get_html(
        exporter: &HtmlExporter,
        results: &[BenchmarkResult],
        unit: Unit,
    ) -> String {
        let html_bytes = exporter
            .serialize(results, Some(unit), SortOrder::MeanTime)
            .expect("HTML export failed");
        String::from_utf8(html_bytes).expect("HTML is not valid UTF-8")
    }

    /// Assert that the HTML has the expected structure
    fn assert_html_structure(html: &str) {
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("<title>Hyperfine Benchmark Results</title>"));
        assert!(html.contains("<div class=\"container\">"));
        assert!(html.contains("function renderSummaryTable()"));
        assert!(html.contains("function renderBoxplot()"));
        assert!(html.contains("font-family: Arial, sans-serif"));
    }

    /// Assert that the HTML contains the benchmark data
    fn assert_contains_benchmark_data(html: &str, results: &[BenchmarkResult]) {
        assert!(html.contains("const benchmarkData ="));
        for result in results {
            assert!(html.contains(&result.command));
        }
    }

    /// Assert unit information in the HTML
    fn assert_unit_information(html: &str, short_name: &str, name: &str, factor: &str) {
        assert!(html.contains(&format!("const unitShortName = \"{}\"", short_name)));
        assert!(html.contains(&format!("const unitName = \"{}\"", name)));
        assert!(html.contains(&format!("const unitFactor = {}", factor)));
    }
}
