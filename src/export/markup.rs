use crate::benchmark::relative_speed::BenchmarkResultWithRelativeSpeed;
use crate::benchmark::{benchmark_result::BenchmarkResult, relative_speed};
use crate::options::SortOrder;
use crate::output::format::format_duration_value;
use crate::util::units::Unit;

use super::Exporter;
use anyhow::Result;

pub enum Alignment {
    Left,
    Right,
}

pub trait MarkupExporter {
    fn table_results(&self, entries: &[BenchmarkResultWithRelativeSpeed], unit: Unit) -> String {
        // prepare table header strings
        let notation = format!("[{}]", unit.short_name());

        // prepare table cells alignment
        let cells_alignment = [
            Alignment::Left,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
            Alignment::Right,
        ];

        // emit table header format
        let mut table = self.table_header(&cells_alignment);

        // emit table header data
        table.push_str(&self.table_row(&[
            "Command",
            &format!("Mean {notation}"),
            &format!("Min {notation}"),
            &format!("Max {notation}"),
            "Relative",
        ]));

        // emit horizontal line
        table.push_str(&self.table_divider(&cells_alignment));

        for entry in entries {
            let measurement = &entry.result;
            // prepare data row strings
            let cmd_str = measurement
                .command_with_unused_parameters
                .replace('|', "\\|");
            let mean_str = format_duration_value(measurement.mean, Some(unit)).0;
            let stddev_str = if let Some(stddev) = measurement.stddev {
                format!(" ± {}", format_duration_value(stddev, Some(unit)).0)
            } else {
                "".into()
            };
            let min_str = format_duration_value(measurement.min, Some(unit)).0;
            let max_str = format_duration_value(measurement.max, Some(unit)).0;
            let rel_str = format!("{:.2}", entry.relative_speed);
            let rel_stddev_str = if entry.is_reference {
                "".into()
            } else if let Some(stddev) = entry.relative_speed_stddev {
                format!(" ± {stddev:.2}")
            } else {
                "".into()
            };

            // prepare table row entries
            table.push_str(&self.table_row(&[
                &self.command(&cmd_str),
                &format!("{mean_str}{stddev_str}"),
                &min_str,
                &max_str,
                &format!("{rel_str}{rel_stddev_str}"),
            ]))
        }

        // emit table footer format
        table.push_str(&self.table_footer(&cells_alignment));

        table
    }

    fn table_row(&self, cells: &[&str]) -> String;

    fn table_divider(&self, cell_aligmnents: &[Alignment]) -> String;

    fn table_header(&self, _cell_aligmnents: &[Alignment]) -> String {
        "".to_string()
    }

    fn table_footer(&self, _cell_aligmnents: &[Alignment]) -> String {
        "".to_string()
    }

    fn command(&self, size: &str) -> String;
}

fn determine_unit_from_results(results: &[BenchmarkResult]) -> Unit {
    if let Some(first_result) = results.first() {
        // Use the first BenchmarkResult entry to determine the unit for all entries.
        format_duration_value(first_result.mean, None).1
    } else {
        // Default to `Second`.
        Unit::Second
    }
}

impl<T: MarkupExporter> Exporter for T {
    fn serialize(
        &self,
        results: &[BenchmarkResult],
        unit: Option<Unit>,
        sort_order: SortOrder,
    ) -> Result<Vec<u8>> {
        let unit = unit.unwrap_or_else(|| determine_unit_from_results(results));
        let entries = relative_speed::compute(results, sort_order);

        let table = self.table_results(&entries, unit);
        Ok(table.as_bytes().to_vec())
    }
}
