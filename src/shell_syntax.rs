//! Conservative detection of shell operators in commands run without a shell.

use std::iter::Peekable;
use std::str::CharIndices;

/// Find the first standalone operator that is neither quoted nor escaped.
///
/// Operators attached to another word (such as `hello>out`), expansions, and
/// other shell syntax are deliberately outside the scope of this check.
pub fn first_unquoted_operator(command: &str) -> Option<&'static str> {
    tokenize(command).find_map(|token| {
        ["&&", "||", "|", ";", ">", ">>", "<"]
            .iter()
            .copied()
            .find(|operator| {
                // An escaped newline joins lines without quoting either side.
                // All other quotes and escapes remain in the token, preventing
                // it from matching an operator.
                token
                    .split("\\\n")
                    .flat_map(str::bytes)
                    .eq(operator.bytes())
            })
    })
}

/// Split words while preserving their original quoting and escaping.
///
/// Word boundaries and comments follow the same rules as `shell_words`, which
/// parses commands for direct execution. This tokenizer neither expands shell
/// expressions nor validates balanced quotes; command parsing handles errors.
fn tokenize(command: &str) -> impl Iterator<Item = &str> {
    Tokens {
        command,
        chars: command.char_indices().peekable(),
    }
}

struct Tokens<'a> {
    command: &'a str,
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> Iterator for Tokens<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // Whitespace, comments, and line continuations cannot start a word.
        let start = loop {
            let &(index, character) = self.chars.peek()?;
            match character {
                ' ' | '\t' | '\n' => {
                    self.chars.next();
                }
                '#' => {
                    for (_, character) in self.chars.by_ref() {
                        if character == '\n' {
                            break;
                        }
                    }
                }
                '\\' if self.chars.clone().nth(1).map(|(_, c)| c) == Some('\n') => {
                    self.chars.next();
                    self.chars.next();
                }
                _ => break index,
            }
        };

        let mut quote = None;
        while let Some((index, character)) = self.chars.next() {
            match (quote, character) {
                (Some('\''), '\'') | (Some('"'), '"') => quote = None,
                (Some('\''), _) => {}
                (_, '\\') => {
                    // Within double quotes, skipping a non-special escaped
                    // character is also safe: it cannot end the quoted word.
                    self.chars.next();
                }
                (Some('"'), _) => {}
                (None, '\'' | '"') => quote = Some(character),
                (None, ' ' | '\t' | '\n') => return Some(&self.command[start..index]),
                _ => {}
            }
        }

        Some(&self.command[start..])
    }
}

#[cfg(test)]
mod tests {
    use super::{first_unquoted_operator, tokenize};

    #[test]
    fn tokenizer_preserves_words_quotes_and_escapes() {
        for (command, expected) in [
            ("", vec![]),
            (" \t\n", vec![]),
            ("  echo\thello\nworld ", vec!["echo", "hello", "world"]),
            (
                r#"echo '' "" 'hello world'"#,
                vec!["echo", "''", "\"\"", "'hello world'"],
            ),
            (
                r#"echo pre'quoted part'"more"post"#,
                vec!["echo", r#"pre'quoted part'"more"post"#],
            ),
            (
                r"echo hello\ world \&\&",
                vec!["echo", r"hello\ world", r"\&\&"],
            ),
            (r#"echo "a\" b" 'a\'"#, vec!["echo", r#""a\" b""#, r"'a\'"]),
            ("echo hello>out", vec!["echo", "hello>out"]),
            (
                "echo café\t&&\n日本語",
                vec!["echo", "café", "&&", "日本語"],
            ),
            ("echo a\r\u{a0}b", vec!["echo", "a\r\u{a0}b"]),
        ] {
            assert_eq!(
                tokenize(command).collect::<Vec<_>>(),
                expected,
                "{command:?}"
            );
        }
    }

    #[test]
    fn tokenizer_handles_comments_and_continuations() {
        for (command, expected) in [
            ("# ignored &&", vec![]),
            (
                "echo # ignored &&\nnext | cat",
                vec!["echo", "next", "|", "cat"],
            ),
            (
                r##"echo a#b \#c "#d" '#e'"##,
                vec!["echo", "a#b", r"\#c", "\"#d\"", "'#e'"],
            ),
            ("\\\n echo \\\n&& more", vec!["echo", "&&", "more"]),
            ("echo hello\\\nworld", vec!["echo", "hello\\\nworld"]),
            ("\\\n# comment\necho", vec!["echo"]),
        ] {
            assert_eq!(
                tokenize(command).collect::<Vec<_>>(),
                expected,
                "{command:?}"
            );
        }
    }

    #[test]
    fn finds_supported_operators_in_command_order() {
        for operator in ["&&", "||", "|", ";", ">", ">>", "<"] {
            let command = format!("echo café\t{operator}\n日本語");
            assert_eq!(first_unquoted_operator(&command), Some(operator));
            assert_eq!(first_unquoted_operator(operator), Some(operator));
        }
        assert_eq!(first_unquoted_operator("echo a >> out && next"), Some(">>"));
        assert_eq!(first_unquoted_operator("echo \\\n&& next"), Some("&&"));
        assert_eq!(first_unquoted_operator("echo &\\\n& next"), Some("&&"));
        assert_eq!(first_unquoted_operator("echo &&\\\n next"), Some("&&"));
        assert_eq!(first_unquoted_operator("echo # &&\nnext | cat"), Some("|"));
        assert_eq!(first_unquoted_operator(r#"echo "&&" || next"#), Some("||"));
        assert_eq!(first_unquoted_operator(r"echo \&\& | cat"), Some("|"));
    }

    #[test]
    fn ignores_quoted_escaped_and_attached_operators() {
        for command in [
            r#"echo "&&""#,
            "echo '&&'",
            r"echo \&\&",
            r"echo &\&",
            r#"echo '&'"&""#,
            r#"echo ""&&"#,
            "echo &&''",
            r#"python -c "print(1 > 0)""#,
            "sh -c 'true && false'",
            r#"sh -c "echo 'a | b' && false""#,
            r#"sh -c 'echo "a | b" && false'"#,
            r#"echo "escaped \" && still quoted""#,
            "echo hello>out",
            "true&&false",
            "echo a;",
            "echo 2> out",
            "echo >>> out",
            "echo $VARIABLE *.rs ~/path NAME=value",
            "echo # && ignored",
            "echo foo#bar",
            "echo\u{a0}&&",
            "echo &&\r",
        ] {
            assert_eq!(first_unquoted_operator(command), None, "{command:?}");
        }
    }

    #[test]
    fn handles_empty_and_malformed_commands_without_panicking() {
        for (command, expected) in [
            ("", vec![]),
            ("\\\n", vec![]),
            ("\\", vec!["\\"]),
            (r"echo \", vec!["echo", r"\"]),
            ("echo 'unclosed &&", vec!["echo", "'unclosed &&"]),
            ("echo \"unclosed &&", vec!["echo", "\"unclosed &&"]),
            ("echo \"unclosed\\", vec!["echo", "\"unclosed\\"]),
        ] {
            assert_eq!(
                tokenize(command).collect::<Vec<_>>(),
                expected,
                "{command:?}"
            );
            assert_eq!(first_unquoted_operator(command), None, "{command:?}");
        }
    }
}
