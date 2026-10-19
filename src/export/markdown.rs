use crate::export::markup::MarkupExporter;

use super::markup::Alignment;

#[derive(Default)]
pub struct MarkdownExporter {}

impl MarkupExporter for MarkdownExporter {
    fn table_row(&self, cells: &[&str]) -> String {
        format!("| {} |\n", cells.join(" | "))
    }

    fn table_divider(&self, cell_aligmnents: &[Alignment]) -> String {
        format!(
            "|{}\n",
            cell_aligmnents
                .iter()
                .map(|a| match a {
                    Alignment::Left => ":---|",
                    Alignment::Right => "---:|",
                })
                .collect::<String>()
        )
    }

    fn command(&self, cmd: &str) -> String {
        let longest_backtick_run = cmd
            .split(|character| character != '`')
            .map(str::len)
            .max()
            .unwrap_or(0);

        if longest_backtick_run == 0 {
            format!("`{cmd}`")
        } else {
            // Use a longer delimiter and keep command backticks separate from it.
            let delimiter = "`".repeat(longest_backtick_run + 1);
            format!("{delimiter} {cmd} {delimiter}")
        }
    }
}

/// Check Markdown-based data row formatting
#[test]
fn test_markdown_formatter_table_data() {
    let formatter = MarkdownExporter::default();

    assert_eq!(formatter.table_row(&["a", "b", "c"]), "| a | b | c |\n");
}

/// Check Markdown-based horizontal line formatting
#[test]
fn test_markdown_formatter_table_divider() {
    let formatter = MarkdownExporter::default();

    let divider = formatter.table_divider(&[Alignment::Left, Alignment::Right, Alignment::Left]);
    assert_eq!(divider, "|:---|---:|:---|\n");
}

#[test]
fn test_markdown_formatter_command_with_backticks() {
    let formatter = MarkdownExporter::default();

    for (command, expected) in [
        ("echo `uname`", "`` echo `uname` ``"),
        ("echo ``quoted``", "``` echo ``quoted`` ```"),
        ("`` `quoted` ``", "``` `` `quoted` `` ```"),
        ("`", "`` ` ``"),
        ("echo ```", "```` echo ``` ````"),
        (" echo `uname` ", "``  echo `uname`  ``"),
    ] {
        assert_eq!(formatter.command(command), expected, "{command:?}");
    }
}
