use super::markup::Alignment;
use crate::export::markup::MarkupExporter;

#[derive(Default)]
pub struct OrgmodeExporter {}

impl MarkupExporter for OrgmodeExporter {
    fn table_row(&self, cells: &[&str]) -> String {
        format!(
            "| {}  |  {} |\n",
            cells.first().unwrap(),
            &cells[1..].join(" |  ")
        )
    }

    fn table_divider(&self, cell_aligmnents: &[Alignment]) -> String {
        format!("|{}--|\n", "--+".repeat(cell_aligmnents.len() - 1))
    }

    fn command(&self, cmd: &str) -> String {
        format!("={cmd}=")
    }
}

/// Check Emacs org-mode data row formatting
#[test]
fn test_orgmode_formatter_table_data() {
    let exporter = OrgmodeExporter::default();

    let actual = exporter.table_row(&["a", "b", "c"]);
    let expect = "| a  |  b |  c |\n";

    assert_eq!(expect, actual);
}

/// Check Emacs org-mode horizontal line formatting
#[test]
fn test_orgmode_formatter_table_line() {
    let exporter = OrgmodeExporter::default();

    let actual = exporter.table_divider(&[
        Alignment::Left,
        Alignment::Left,
        Alignment::Left,
        Alignment::Left,
        Alignment::Left,
    ]);
    let expect = "|--+--+--+--+--|\n";

    assert_eq!(expect, actual);
}
