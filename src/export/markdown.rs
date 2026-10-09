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
        // Code spans render line endings as spaces, but table rows cannot contain them.
        let cmd = cmd.replace("\r\n", " ").replace(['\r', '\n'], " ");
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

#[test]
fn test_markdown_formatter_command_with_line_endings() {
    let formatter = MarkdownExporter::default();

    for (command, expected) in [
        ("echo one\necho two", "`echo one echo two`"),
        ("echo one\recho two", "`echo one echo two`"),
        ("echo one\r\necho two", "`echo one echo two`"),
        ("echo one\n\necho two", "`echo one  echo two`"),
        ("echo one\r\n\r\necho two", "`echo one  echo two`"),
        ("echo `uname`\ncat", "`` echo `uname` cat ``"),
        ("\necho\n", "` echo `"),
        ("echo\r", "`echo `"),
    ] {
        let formatted = formatter.command(command);
        assert_eq!(formatted, expected, "{command:?}");
        let row = formatter.table_row(&[&formatted, "1.000", "", ""]);
        assert_eq!(row.lines().count(), 1, "{command:?}");
    }
}
