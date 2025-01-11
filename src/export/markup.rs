use crate::benchmark::relative_speed::BenchmarkResultWithRelativeSpeed;
use crate::benchmark::{benchmark_result::BenchmarkResult, relative_speed};
use crate::options::SortOrder;
use crate::quantity::{TimeQuantity, TimeUnit};

use super::Exporter;
use anyhow::Result;

pub enum Alignment {
    Left,
    Right,
}

pub trait MarkupExporter {
    fn table_results(
        &self,
        entries: &[BenchmarkResultWithRelativeSpeed],
        time_unit: TimeUnit,
    ) -> String {
        // prepare table header strings
        let notation = format!("[{}]", time_unit.short_name());

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
            let result = &entry.result;
            // prepare data row strings
            let cmd_str = result.command_with_unused_parameters().replace('|', "\\|");
            let mean_str = result.mean_wall_clock_time().format_value(time_unit);
            let stddev_str = if let Some(stddev) = result.measurements.stddev() {
                format!(" ± {}", stddev.format_value(time_unit))
            } else {
                "".into()
            };
            let min_str = result.measurements.min().format_value(time_unit);
            let max_str = result.measurements.max().format_value(time_unit);
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

fn determine_unit_from_results(results: &[BenchmarkResult]) -> TimeUnit {
    if let Some(first_result) = results.first() {
        // Use the first BenchmarkResult entry to determine the unit for all entries.
        first_result.mean_wall_clock_time().suitable_unit()
    } else {
        // Default to `Second`.
        TimeUnit::Second
    }
}

impl<T: MarkupExporter> Exporter for T {
    fn serialize(
        &self,
        results: &[BenchmarkResult],
        time_unit: Option<TimeUnit>,
        sort_order: SortOrder,
    ) -> Result<Vec<u8>> {
        let unit = time_unit.unwrap_or_else(|| determine_unit_from_results(results));
        let entries = relative_speed::compute(results, sort_order);

        let table = self.table_results(&entries, unit);
        Ok(table.as_bytes().to_vec())
    }
}
