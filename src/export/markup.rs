use crate::benchmark::relative_speed::BenchmarkResultWithRelativeSpeed;
use crate::output::format::format_duration_value;
use crate::util::units::Unit;

pub trait MarkupFormatter {
    fn table_results(&self, entries: &[BenchmarkResultWithRelativeSpeed], unit: Unit) -> String {
        // prepare table header strings
        let notation = format!("[{}]", unit.short_name());
        let head: [&str; 5] = [
            "Command",
            &format!("Mean {}", notation),
            &format!("Min {}", notation),
            &format!("Max {}", notation),
            "Relative",
        ];

        // emit header
        let mut table = self.table_data(&head);

        // emit horizontal line
        table.push_str(&self.table_line(head.len()));

        for entry in entries {
            let measurement = &entry.result;
            // prepare data row strings
            let cmd_str = measurement.command.replace('|', "\\|");
            let mean_str = format_duration_value(measurement.mean, Some(unit)).0;
            let stddev_str = if let Some(stddev) = measurement.stddev {
                format!(" ± {}", format_duration_value(stddev, Some(unit)).0)
            } else {
                "".into()
            };
            let min_str = format_duration_value(measurement.min, Some(unit)).0;
            let max_str = format_duration_value(measurement.max, Some(unit)).0;
            let rel_str = format!("{:.2}", entry.relative_speed);
            let rel_stddev_str = if entry.is_fastest {
                "".into()
            } else if let Some(stddev) = entry.relative_speed_stddev {
                format!(" ± {:.2}", stddev)
            } else {
                "".into()
            };
            // prepare table row entries
            let data: [&str; 5] = [
                &self.command(&cmd_str),
                &format!("{}{}", mean_str, stddev_str),
                &min_str,
                &max_str,
                &format!("{}{}", rel_str, rel_stddev_str),
            ];
            table.push_str(&self.table_data(&data))
        }

        table
    }

    fn table_data(&self, data: &[&str]) -> String;

    fn table_line(&self, size: usize) -> String;

    fn command(&self, size: &str) -> String;
}
