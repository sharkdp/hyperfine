use rust_decimal::Decimal;
/// This module contains common internal types.
use serde::*;
use std::fmt;

use crate::hyperfine::units::{Second, Unit};

#[cfg(not(windows))]
pub const DEFAULT_SHELL: &str = "sh";

#[cfg(windows)]
pub const DEFAULT_SHELL: &str = "cmd.exe";

#[derive(Debug, Clone, Serialize, Copy, PartialEq)]
#[serde(untagged)]
pub enum NumericType {
    Int(i32),
    Decimal(Decimal),
}

impl fmt::Display for NumericType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            NumericType::Int(i) => fmt::Display::fmt(&i, f),
            NumericType::Decimal(i) => fmt::Display::fmt(&i, f),
        }
    }
}

impl Into<NumericType> for i32 {
    fn into(self) -> NumericType {
        NumericType::Int(self)
    }
}

impl Into<NumericType> for Decimal {
    fn into(self) -> NumericType {
        NumericType::Decimal(self)
    }
}

/// A command that should be benchmarked.
#[derive(Debug, Clone)]
pub struct Command<'a> {
    /// The command that should be executed (without parameter substitution)
    expression: &'a str,

    /// A possible parameter value.
    parameter: Option<(&'a str, NumericType)>,
}

impl<'a> Command<'a> {
    pub fn new(expression: &'a str) -> Command<'a> {
        Command {
            expression,
            parameter: None,
        }
    }

    pub fn new_parametrized(
        expression: &'a str,
        parameter: &'a str,
        value: NumericType,
    ) -> Command<'a> {
        Command {
            expression,
            parameter: Some((parameter, value)),
        }
    }

    pub fn get_shell_command(&self) -> String {
        match self.parameter {
            Some((param_name, param_value)) => self.expression.replace(
                &format!("{{{param_name}}}", param_name = param_name),
                &param_value.to_string(),
            ),
            None => self.expression.into(),
        }
    }

    pub fn get_parameter(&self) -> Option<(&'a str, NumericType)> {
        self.parameter
    }
}

impl<'a> fmt::Display for Command<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_shell_command())
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
    pub shell: String,

    /// Forward benchmark's stdout to hyperfine's stdout
    pub show_output: bool,

    /// Which time unit to use for CLI & Markdown output
    pub time_unit: Option<Unit>,
}

impl Default for HyperfineOptions {
    fn default() -> HyperfineOptions {
        HyperfineOptions {
            warmup_count: 0,
            runs: Runs::default(),
            min_time_sec: 3.0,
            failure_action: CmdFailureAction::RaiseError,
            preparation_command: None,
            cleanup_command: None,
            output_style: OutputStyleOption::Full,
            shell: DEFAULT_SHELL.to_string(),
            show_output: false,
            time_unit: None,
        }
    }
}

/// Set of values that will be exported.
#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct BenchmarkResult {
    /// The command that was run
    pub command: String,

    /// The mean run time
    pub mean: Second,

    /// The standard deviation of all run times
    pub stddev: Second,

    /// The median run time
    pub median: Second,

    /// Time spend in user space
    pub user: Second,

    /// Time spent in system space
    pub system: Second,

    /// Min time measured
    pub min: Second,

    /// Max time measured
    pub max: Second,

    /// All run time measurements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times: Option<Vec<Second>>,

    /// Any parameter used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter: Option<NumericType>,
}

impl BenchmarkResult {
    /// Create a new entry with the given values.
    pub fn new(
        command: String,
        mean: Second,
        stddev: Second,
        median: Second,
        user: Second,
        system: Second,
        min: Second,
        max: Second,
        times: Vec<Second>,
        parameter: Option<NumericType>,
    ) -> Self {
        BenchmarkResult {
            command,
            mean,
            stddev,
            median,
            user,
            system,
            min,
            max,
            times: Some(times),
            parameter,
        }
    }
}
