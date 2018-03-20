use super::{ExportEntry, Exporter};

use std::io::Result;

const MULTIPLIER: f64 = 1e3;

#[derive(Default)]
pub struct MarkdownExporter {}

impl Exporter for MarkdownExporter {
    fn serialize(&self, results: &Vec<ExportEntry>) -> Result<Vec<u8>> {
        let mut destination = start_table();

        for result in results {
            add_table_row(&mut destination, result);
        }

        Ok(destination)
    }
}

fn start_table() -> Vec<u8> {
    "| Benchmark | Mean [ms] | Min. [ms] | Max. [ms] |\n|----|----|----|----|\n"
        .bytes()
        .collect()
}
fn add_table_row(dest: &mut Vec<u8>, entry: &ExportEntry) {
    dest.extend(
        format!(
            "| `{}` | {:.1} ± {:.1} | {:.1} | {:.1} |\n",
            entry.command.replace("|", "\\|"),
            entry.mean * MULTIPLIER,
            entry.stddev * MULTIPLIER,
            entry.min * MULTIPLIER,
            entry.max * MULTIPLIER
        ).as_bytes(),
    );
}
