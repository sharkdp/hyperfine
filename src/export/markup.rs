use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::metric::{MetricSelection, Stats, Unit};

use super::Exporter;
use anyhow::{Context, Result};

pub enum Alignment {
    Left,
    Right,
}

pub trait MarkupExporter {
    fn table_results(
        &self,
        results: &[BenchmarkResult],
        summaries: &[Stats],
        primary: MetricSelection,
        unit: Unit,
    ) -> String {
        let notation = format!("[{}]", unit.symbol);
        let cells_alignment = [
            Alignment::Left,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
        ];
        let mut table = self.table_header(&cells_alignment);
        table.push_str(&self.table_row(&[
            "Command",
            &format!("Mean {} {notation}", primary.metric.label()),
            &format!("Min {notation}"),
            &format!("Max {notation}"),
            "Change",
        ]));
        table.push_str(&self.table_divider(&cells_alignment));

        for (index, (result, stats)) in results.iter().zip(summaries).enumerate() {
            let command = result.display_name.replace('|', "\\|");
            let mean = unit.format_value(stats.mean);
            let stddev = stats
                .stddev
                .map(|value| format!(" ± {}", unit.format_value(value)))
                .unwrap_or_default();
            let change = if index == 0 {
                "reference".to_owned()
            } else {
                stats.format_change_from(&summaries[0])
            };
            table.push_str(&self.table_row(&[
                &self.command(&command),
                &format!("{mean}{stddev}"),
                &unit.format_value(stats.min),
                &unit.format_value(stats.max),
                &change,
            ]));
        }
        table.push_str(&self.table_footer(&cells_alignment));
        table
    }

    fn table_row(&self, cells: &[&str]) -> String;

    fn table_divider(&self, cell_alignments: &[Alignment]) -> String;

    fn table_header(&self, _cell_alignments: &[Alignment]) -> String {
        String::new()
    }

    fn table_footer(&self, _cell_alignments: &[Alignment]) -> String {
        String::new()
    }

    fn command(&self, command: &str) -> String;
}

impl<T: MarkupExporter> Exporter for T {
    fn serialize(&self, results: &[BenchmarkResult], primary: MetricSelection) -> Result<Vec<u8>> {
        let stats: Vec<_> = results
            .iter()
            .map(|result| {
                primary
                    .metric
                    .summarize(&result.measurements)
                    .with_context(|| {
                        format!(
                            "Metric '{}' is unavailable for '{}'",
                            primary.metric.name(),
                            result.get_name()
                        )
                    })
            })
            .collect::<Result<_>>()?;
        // Use one common unit, avoiding an unscaled column if the reference is zero.
        let mean = stats
            .iter()
            .find(|stats| stats.mean != 0.0)
            .map_or(0.0, |stats| stats.mean);
        let unit = primary.display_unit(mean);
        Ok(self
            .table_results(results, &stats, primary, unit)
            .into_bytes())
    }
}
