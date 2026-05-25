//! Helpers for parsing Windows command lines.
//!
//! Unlike `shell_words::split`, backslashes are not interpreted as escape
//! characters. This preserves paths such as `Downloads\in.exe` when running
//! commands with `--shell=none`.

/// Split a command line on whitespace outside of double quotes.
///
/// Backslashes are treated literally, which matches Windows path semantics for
/// raw command execution.
pub fn split_command_line(command_line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_double_quotes = false;

    for ch in command_line.chars() {
        match ch {
            '"' => {
                in_double_quotes = !in_double_quotes;
                current.push(ch);
            }
            ' ' | '\t' if !in_double_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::split_command_line;

    #[test]
    fn preserves_backslashes_in_paths() {
        assert_eq!(
            split_command_line(r"Downloads\in.exe"),
            vec!["Downloads\\in.exe".to_string()]
        );
    }

    #[test]
    fn preserves_drive_paths_with_arguments() {
        assert_eq!(
            split_command_line(r"C:\Users\wes\tool.exe --version"),
            vec![
                "C:\\Users\\wes\\tool.exe".to_string(),
                "--version".to_string(),
            ]
        );
    }

    #[test]
    fn respects_double_quotes() {
        assert_eq!(
            split_command_line(r#"echo "a b" >> C:\out\file.log"#),
            vec![
                "echo".to_string(),
                "\"a b\"".to_string(),
                ">>".to_string(),
                "C:\\out\\file.log".to_string(),
            ]
        );
    }
}
