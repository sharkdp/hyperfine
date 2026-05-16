//! Helpers for running commands through `cmd.exe` on Windows.
//!
//! `cmd.exe` treats `/` as a switch prefix, so paths like `./target/debug/foo.exe`
//! fail unless forward slashes are normalized to backslashes.

/// Normalize a command line for execution via `cmd.exe /C`.
///
/// Path-like arguments that use `/` as a separator are converted to `\`.
/// URLs (`://`) and other tokens are left unchanged.
#[cfg(windows)]
pub fn normalize_command_line_for_cmd(command_line: &str) -> String {
    normalize_command_line_for_cmd_impl(command_line)
}

#[cfg(not(windows))]
pub fn normalize_command_line_for_cmd(command_line: &str) -> String {
    command_line.to_string()
}

#[cfg_attr(not(windows), allow(dead_code))]
fn normalize_command_line_for_cmd_impl(command_line: &str) -> String {
    let Ok(words) = shell_words::split(command_line) else {
        return command_line.to_string();
    };

    let normalized: Vec<String> = words
        .into_iter()
        .map(|word| {
            if should_normalize_path_token(&word) {
                word.replace('/', "\\")
            } else {
                word
            }
        })
        .collect();

    join_for_cmd(&normalized)
}

#[cfg_attr(not(windows), allow(dead_code))]
fn join_for_cmd(words: &[String]) -> String {
    words
        .iter()
        .map(|word| {
            if word.chars().any(char::is_whitespace) {
                format!("\"{}\"", word.replace('"', "\\\""))
            } else {
                word.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg_attr(not(windows), allow(dead_code))]
fn should_normalize_path_token(token: &str) -> bool {
    if token.contains("://") || !token.contains('/') {
        return false;
    }

    if token.starts_with("./") || token.starts_with("../") {
        return true;
    }

    if token.len() >= 3 {
        let bytes = token.as_bytes();
        if bytes[1] == b':' && bytes[2] == b'/' && bytes[0].is_ascii_alphabetic() {
            return true;
        }
    }

    has_windows_executable_extension(token)
}

#[cfg_attr(not(windows), allow(dead_code))]
fn has_windows_executable_extension(token: &str) -> bool {
    const EXTENSIONS: &[&str] = &[
        ".exe", ".bat", ".cmd", ".com", ".msi", ".ps1", ".vbs", ".dll",
    ];

    let lower = token.to_ascii_lowercase();
    EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

#[cfg(test)]
mod tests {
    use super::normalize_command_line_for_cmd_impl;

    #[test]
    fn normalizes_relative_paths() {
        assert_eq!(
            normalize_command_line_for_cmd_impl("./target/debug/foo.exe"),
            ".\\target\\debug\\foo.exe"
        );
    }

    #[test]
    fn normalizes_drive_paths() {
        assert_eq!(
            normalize_command_line_for_cmd_impl("C:/Users/foo/bar.exe"),
            "C:\\Users\\foo\\bar.exe"
        );
    }

    #[test]
    fn leaves_urls_unchanged() {
        let url = "curl https://example.com/path";
        assert_eq!(normalize_command_line_for_cmd_impl(url), url);
    }

    #[test]
    fn normalizes_only_path_tokens_in_multi_arg_commands() {
        assert_eq!(
            normalize_command_line_for_cmd_impl(r#"./foo.exe --output "out/a.txt""#),
            r".\foo.exe --output out/a.txt"
        );
    }

    #[test]
    fn leaves_non_path_tokens_unchanged() {
        assert_eq!(
            normalize_command_line_for_cmd_impl("echo hello/world"),
            "echo hello/world"
        );
    }
}
