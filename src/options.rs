use std::fmt;
use std::process::Command;

use crate::error::OptionsError;
use crate::units::{Second, Unit};

#[cfg(not(windows))]
pub const DEFAULT_SHELL: &str = "sh";

#[cfg(windows)]
pub const DEFAULT_SHELL: &str = "cmd.exe";

/// Shell to use for executing benchmarked commands
#[derive(Debug)]
pub enum Shell {
    /// Default shell command
    Default(&'static str),
    /// Custom shell command specified via --shell
    Custom(Vec<String>),
}

impl Default for Shell {
    fn default() -> Self {
        Shell::Default(DEFAULT_SHELL)
    }
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shell::Default(cmd) => write!(f, "{}", cmd),
            Shell::Custom(cmdline) => write!(f, "{}", shell_words::join(cmdline)),
        }
    }
}

impl Shell {
    /// Parse given string as shell command line
    pub fn parse<'a>(s: &str) -> Result<Self, OptionsError<'a>> {
        let v = shell_words::split(s).map_err(OptionsError::ShellParseError)?;
        if v.is_empty() || v[0].is_empty() {
            return Err(OptionsError::EmptyShell);
        }
        Ok(Shell::Custom(v))
    }

    pub fn command(&self) -> Command {
        match self {
            Shell::Default(cmd) => Command::new(cmd),
            Shell::Custom(cmdline) => {
                let mut c = Command::new(&cmdline[0]);
                c.args(&cmdline[1..]);
                c
            }
        }
    }
}

/// Action to take when an executed command fails.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CmdFailureAction {
    /// Exit with an error message
    RaiseError,

    /// Simply ignore the non-zero exit code
    Ignore,
}

/// Output style type option
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputStyleOption {
    /// Do not output with colors or any special formatting
    Basic,

    /// Output with full color and formatting
    Full,

    /// Keep elements such as progress bar, but use no coloring
    NoColor,

    /// Keep coloring, but use no progress bar
    Color,

    /// Disable all the output
    Disabled,
}

/// Number of runs for a benchmark
pub struct Runs {
    /// Minimum number of benchmark runs
    pub min: u64,

    /// Maximum number of benchmark runs
    pub max: Option<u64>,
}

impl Default for Runs {
    fn default() -> Runs {
        Runs { min: 10, max: None }
    }
}

/// A set of options for hyperfine
pub struct HyperfineOptions {
    /// Number of warmup runs
    pub warmup_count: u64,

    /// Number of benchmark runs
    pub runs: Runs,

    /// Minimum benchmarking time
    pub min_time_sec: Second,

    /// Whether or not to ignore non-zero exit codes
    pub failure_action: CmdFailureAction,

    /// Command to run before each timing run
    pub preparation_command: Option<Vec<String>>,

    /// Command to run after each benchmark
    pub cleanup_command: Option<String>,

    /// What color mode to use for output
    pub output_style: OutputStyleOption,

    /// The shell to use for executing commands.
    pub shell: Shell,

    /// Forward benchmark's stdout to hyperfine's stdout
    pub show_output: bool,

    /// Which time unit to use for CLI & Markdown output
    pub time_unit: Option<Unit>,

    /// A list of custom command names that, if defined,
    /// will be used instead of the command itself in
    /// benchmark outputs.
    pub names: Option<Vec<String>>,
}

impl Default for HyperfineOptions {
    fn default() -> HyperfineOptions {
        HyperfineOptions {
            names: None,
            warmup_count: 0,
            runs: Runs::default(),
            min_time_sec: 3.0,
            failure_action: CmdFailureAction::RaiseError,
            preparation_command: None,
            cleanup_command: None,
            output_style: OutputStyleOption::Full,
            shell: Shell::default(),
            show_output: false,
            time_unit: None,
        }
    }
}

#[test]
fn test_shell_default_command() {
    let shell = Shell::default();

    let s = format!("{}", shell);
    assert_eq!(&s, DEFAULT_SHELL);

    let cmd = shell.command();
    // Command::get_program is not yet available in stable channel.
    // https://doc.rust-lang.org/std/process/struct.Command.html#method.get_program
    let s = format!("{:?}", cmd);
    assert_eq!(s, format!("\"{}\"", DEFAULT_SHELL));
}

#[test]
fn test_shell_parse_command() {
    let shell = Shell::parse("shell -x 'aaa bbb'").unwrap();

    let s = format!("{}", shell);
    assert_eq!(&s, "shell -x 'aaa bbb'");

    let cmd = shell.command();
    // Command::get_program and Command::args are not yet available in stable channel.
    // https://doc.rust-lang.org/std/process/struct.Command.html#method.get_program
    let s = format!("{:?}", cmd);
    assert_eq!(&s, r#""shell" "-x" "aaa bbb""#);

    // Error cases

    match Shell::parse("shell 'foo").unwrap_err() {
        OptionsError::ShellParseError(_) => { /* ok */ }
        e => assert!(false, "Unexpected error: {}", e),
    }

    match Shell::parse("").unwrap_err() {
        OptionsError::EmptyShell => { /* ok */ }
        e => assert!(false, "Unexpected error: {}", e),
    }

    match Shell::parse("''").unwrap_err() {
        OptionsError::EmptyShell => { /* ok */ }
        e => assert!(false, "Unexpected error: {}", e),
    }
}
