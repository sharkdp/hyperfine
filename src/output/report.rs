use std::io::{self, Write};

use anyhow::{ensure, Result};
use colored::Colorize;

use crate::benchmark::{benchmark_result::BenchmarkResult, measurement::Measurements};
use crate::metric::MetricSelection;
use crate::output::console_writeln;

// Reserve enough space for typical values and every supported unit suffix so
// independently printed benchmarks keep the same layout.
const MIN_VALUE_WIDTH: usize = 7;
const MIN_UNIT_WIDTH: usize = 3;
const MIN_CHANGE_WIDTH: usize = 12;

/// Print a completed benchmark immediately, comparing every metric with benchmark 1.
pub fn print(
    number: usize,
    name: &str,
    measurements: &Measurements,
    metrics: &[MetricSelection],
    skip_unavailable_metrics: bool,
    reference: Option<&BenchmarkResult>,
) -> Result<()> {
    let comparison = reference.is_some();
    let multiple_runs = measurements.len() > 1;
    let value_columns = 1..if multiple_runs { 5 } else { 2 };
    let mut header = vec![
        String::new(),
        "mean".to_owned(),
        "σ".to_owned(),
        "min".to_owned(),
        "max".to_owned(),
    ];
    if !multiple_runs {
        header.truncate(2);
        header[1] = "Value".to_owned();
    }
    if comparison {
        header.push(String::new());
    }
    let mut rows = vec![(header, "")];
    for selection in metrics {
        let Some(summary) = selection.metric.summarize(measurements) else {
            ensure!(
                skip_unavailable_metrics,
                "Metric '{}' is unavailable for '{name}'",
                selection.metric.name()
            );
            continue;
        };
        let baseline = reference.and_then(|r| selection.metric.summarize(&r.measurements));
        // Keep auto units consistent with the first command for easier comparison.
        let scale_mean = baseline
            .as_ref()
            .filter(|stats| stats.mean != 0.0)
            .unwrap_or(&summary)
            .mean;
        let unit = selection.display_unit(scale_mean);
        let mut row = vec![
            selection.metric.label().to_owned(),
            unit.format_value(summary.mean),
        ];
        if let Some(stddev) = summary.stddev {
            row.push(unit.format_value(stddev));
            row.push(unit.format_value(summary.min));
            row.push(unit.format_value(summary.max));
        }
        if comparison {
            row.push(baseline.as_ref().map_or_else(
                || "N/A".to_owned(),
                |baseline| summary.format_change_from(baseline),
            ));
        }
        rows.push((
            row,
            if unit.symbol == "count" {
                ""
            } else {
                unit.symbol
            },
        ));
    }
    // Align numbers and unit suffixes independently so mixed units do not shift
    // the values or the ± and … separators.
    let unit_width = rows
        .iter()
        .map(|(_, unit)| unit.chars().count())
        .max()
        .unwrap()
        .max(MIN_UNIT_WIDTH);
    // Keep labels aligned across benchmarks even when some metrics are skipped.
    let metric_width = metrics
        .iter()
        .map(|selection| selection.metric.label().chars().count())
        .max()
        .unwrap_or(0);
    let widths: Vec<_> = (0..rows[0].0.len())
        .map(|column| {
            let width = rows
                .iter()
                .map(|(row, _)| row[column].chars().count())
                .max()
                .unwrap();
            if column == 0 {
                width.max(metric_width)
            } else if value_columns.contains(&column) {
                width.max(MIN_VALUE_WIDTH)
            } else {
                width.max(MIN_CHANGE_WIDTH)
            }
        })
        .collect();
    let mut stdout = io::stdout().lock();
    console_writeln!(
        stdout,
        "{}: {} {}",
        format!("Benchmark {}", number + 1).bold(),
        name.white().bold(),
        format!(
            "({} {})",
            measurements.len(),
            if measurements.len() == 1 {
                "run"
            } else {
                "runs"
            }
        )
        .dimmed()
    )?;
    for (index, (row, unit)) in rows.iter().enumerate() {
        let mut line = String::new();
        for (column, value) in row.iter().enumerate() {
            let is_change = comparison && column == row.len() - 1;
            if index == 0 && is_change {
                break;
            }
            if column > 0 {
                line.push_str(match (multiple_runs, column) {
                    (true, 2) => " ± ",
                    (true, 4) => " … ",
                    _ => "  ",
                });
            }
            let padding = " ".repeat(widths[column] - value.chars().count());
            let has_unit = value_columns.contains(&column);
            let text = if has_unit && !unit.is_empty() {
                format!("{value} {unit}")
            } else {
                value.clone()
            };
            let styled = if is_change {
                if value.starts_with('-') {
                    text.green().to_string()
                } else if value.starts_with('+') {
                    text.red().to_string()
                } else {
                    text.normal().to_string()
                }
            } else if column == 1 || column == 2 {
                text.green().to_string()
            } else if column == 3 {
                text.cyan().to_string()
            } else if column == 4 {
                text.purple().to_string()
            } else {
                text.normal().to_string()
            };
            if column == 0 {
                line.push_str(&format!("{styled}{padding}"));
            } else {
                line.push_str(&format!("{padding}{styled}"));
            }
            if has_unit && unit_width > 0 {
                // Empty suffixes (headers and unscaled counters) also need the
                // space normally separating a number from its unit.
                let unit_padding = unit_width - unit.chars().count() + usize::from(unit.is_empty());
                line.push_str(&" ".repeat(unit_padding));
            }
        }
        let line = line.trim_end();
        console_writeln!(
            stdout,
            "  {}",
            if index == 1 {
                line.bold()
            } else {
                line.normal()
            }
        )?;
    }
    Ok(())
}
