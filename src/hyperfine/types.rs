use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
/// This module contains common internal types.
use serde::*;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;

use crate::hyperfine::units::{Second, Unit};

#[cfg(not(windows))]
pub const DEFAULT_SHELL: &str = "sh";

#[cfg(windows)]
pub const DEFAULT_SHELL: &str = "cmd.exe";

#[derive(Debug, Clone, Serialize, Copy, PartialEq, Eq)]
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

impl TryFrom<NumericType> for usize {
    type Error = ();

    fn try_from(numeric: NumericType) -> Result<Self, Self::Error> {
        match numeric {
            NumericType::Int(i) => usize::try_from(i).map_err(|_| ()),
            NumericType::Decimal(d) => match d.to_u64() {
                Some(u) => usize::try_from(u).map_err(|_| ()),
                None => Err(()),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterValue {
    Text(String),
    Numeric(NumericType),
}

impl<'a> ToString for ParameterValue {
    fn to_string(&self) -> String {
        match self {
            ParameterValue::Text(ref value) => value.clone(),
            ParameterValue::Numeric(value) => value.to_string(),
        }
    }
}

/// A command that should be benchmarked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command<'a> {
    /// The command name (without parameter substitution)
    name: Option<&'a str>,

    /// The command that should be executed (without parameter substitution)
    expression: &'a str,

    /// Zero or more parameter values.
    parameters: Vec<(&'a str, ParameterValue)>,
}

impl<'a> Command<'a> {
    pub fn new(name: Option<&'a str>, expression: &'a str) -> Command<'a> {
        Command {
            name,
            expression,
            parameters: Vec::new(),
        }
    }

    pub fn new_parametrized(
        name: Option<&'a str>,
        expression: &'a str,
        parameters: Vec<(&'a str, ParameterValue)>,
    ) -> Command<'a> {
        Command {
            name,
            expression,
            parameters,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.map_or_else(
            || self.get_shell_command(),
            |name| self.replace_parameters_in(name),
        )
    }

    pub fn get_shell_command(&self) -> String {
        self.replace_parameters_in(self.expression)
    }

    pub fn get_parameters(&self) -> &Vec<(&'a str, ParameterValue)> {
        &self.parameters
    }

    fn replace_parameters_in(&self, original: &str) -> String {
        let mut result = String::new();
        let mut replacements = BTreeMap::<String, String>::new();
        for (param_name, param_value) in &self.parameters {
            replacements.insert(
                format!("{{{param_name}}}", param_name = param_name),
                param_value.to_string(),
            );
        }
        let mut remaining = original;
        // Manually replace consecutive occurrences to avoid double-replacing: e.g.,
        //
        //     hyperfine -L foo 'a,{bar}' -L bar 'baz,quux' 'echo {foo} {bar}'
        //
        // should not ever run 'echo baz baz'. See `test_get_shell_command_nonoverlapping`.
        'outer: while let Some(head) = remaining.chars().next() {
            for (k, v) in &replacements {
                if remaining.starts_with(k.as_str()) {
                    result.push_str(&v);
                    remaining = &remaining[k.len()..];
                    continue 'outer;
                }
            }
            result.push(head);
            remaining = &remaining[head.len_utf8()..];
        }
        result
    }
}

#[test]
fn test_get_shell_command_nonoverlapping() {
    let cmd = Command::new_parametrized(
        None,
        "echo {foo} {bar}",
        vec![
            ("foo", ParameterValue::Text("{bar} baz".into())),
            ("bar", ParameterValue::Text("quux".into())),
        ],
    );
    assert_eq!(cmd.get_shell_command(), "echo {bar} baz quux");
}

#[test]
fn test_get_parameterized_command_name() {
    let cmd = Command::new_parametrized(
        Some("name-{bar}-{foo}"),
        "echo {foo} {bar}",
        vec![
            ("foo", ParameterValue::Text("baz".into())),
            ("bar", ParameterValue::Text("quux".into())),
        ],
    );
    assert_eq!(cmd.get_name(), "name-quux-baz");
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
            shell: DEFAULT_SHELL.to_string(),
            show_output: false,
            time_unit: None,
        }
    }
}

/// Set of values that will be exported.
// NOTE: `serde` is used for JSON serialization, but not for CSV serialization due to the
// `parameters` map. Update `src/hyperfine/export/csv.rs` with new fields, as appropriate.
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

    /// Any parameter values used
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, String>,
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
        parameters: BTreeMap<String, String>,
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
            parameters,
        }
    }
}
