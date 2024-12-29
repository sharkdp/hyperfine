use std::fs::File;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{cmp, env, fmt, io};

use anyhow::ensure;
use clap::ArgMatches;

use crate::command::Commands;
use crate::error::OptionsError;
use crate::util::units::{Second, Unit};

use anyhow::Result;

#[cfg(not(windows))]
pub const DEFAULT_SHELL: &str = "sh";

#[cfg(windows)]
pub const DEFAULT_SHELL: &str = "cmd.exe";

/// Shell to use for executing benchmarked commands
#[derive(Debug, PartialEq)]
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
            Shell::Default(cmd) => write!(f, "{cmd}"),
            Shell::Custom(cmdline) => write!(f, "{}", shell_words::join(cmdline)),
        }
    }
}

impl Shell {
    /// Parse given string as shell command line
    pub fn parse_from_str<'a>(s: &str) -> Result<Self, OptionsError<'a>> {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdFailureAction {
    /// Exit with an error message
    RaiseError,

    /// Simply ignore the non-zero exit code
    Ignore,
}

/// Output style type option
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Command,
    MeanTime,
}

/// Bounds for the number of benchmark runs
pub struct RunBounds {
    /// Minimum number of benchmark runs
    pub min: u64,

    /// Maximum number of benchmark runs
    pub max: Option<u64>,
}

impl Default for RunBounds {
    fn default() -> Self {
        RunBounds { min: 10, max: None }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum CommandInputPolicy {
    /// Read from the null device
    #[default]
    Null,

    /// Read input from a file
    File(PathBuf),
}

impl CommandInputPolicy {
    pub fn get_stdin(&self) -> io::Result<Stdio> {
        let stream: Stdio = match self {
            CommandInputPolicy::Null => Stdio::null(),

            CommandInputPolicy::File(path) => {
                let file: File = File::open(path)?;
                Stdio::from(file)
            }
        };

        Ok(stream)
    }
}

/// How to handle the output of benchmarked commands
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CommandOutputPolicy {
    /// Redirect output to the null device
    #[default]
    Null,

    /// Feed output through a pipe before discarding it
    Pipe,

    /// Redirect output to a file
    File(PathBuf),

    /// Show command output on the terminal
    Inherit,
}

impl CommandOutputPolicy {
    pub fn get_stdout_stderr(&self) -> io::Result<(Stdio, Stdio)> {
        let streams = match self {
            CommandOutputPolicy::Null => (Stdio::null(), Stdio::null()),

            // Typically only stdout is performance-relevant, so just pipe that
            CommandOutputPolicy::Pipe => (Stdio::piped(), Stdio::null()),

            CommandOutputPolicy::File(path) => {
                let file = File::create(path)?;
                (file.into(), Stdio::null())
            }

            CommandOutputPolicy::Inherit => (Stdio::inherit(), Stdio::inherit()),
        };

        Ok(streams)
    }
}

#[derive(Debug, PartialEq)]
pub enum ExecutorKind {
    Raw,
    Shell(Shell),
    Mock(Option<String>),
}

impl Default for ExecutorKind {
    fn default() -> Self {
        ExecutorKind::Shell(Shell::default())
    }
}

/// The main settings for a hyperfine benchmark session
pub struct Options {
    /// Upper and lower bound for the number of benchmark runs
    pub run_bounds: RunBounds,

    /// Number of warmup runs
    pub warmup_count: u64,

    /// Minimum benchmarking time
    pub min_benchmarking_time: Second,

    /// Whether or not to ignore non-zero exit codes
    pub command_failure_action: CmdFailureAction,

    // Command to use as a reference for relative speed comparison
    pub reference_command: Option<String>,

    /// Command(s) to run before each timing run
    pub preparation_command: Option<Vec<String>>,

    /// Command(s) to run after each timing run
    pub conclusion_command: Option<Vec<String>>,

    /// Command to run before each *batch* of timing runs, i.e. before each individual benchmark
    pub setup_command: Option<String>,

    /// Command to run after each *batch* of timing runs, i.e. after each individual benchmark
    pub cleanup_command: Option<String>,

    /// What color mode to use for the terminal output
    pub output_style: OutputStyleOption,

    /// How to order benchmarks in the relative speed comparison
    pub sort_order_speed_comparison: SortOrder,

    /// How to order benchmarks in the markup format exports
    pub sort_order_exports: SortOrder,

    /// Determines how we run commands
    pub executor_kind: ExecutorKind,

    /// Where input to the benchmarked command comes from
    pub command_input_policy: CommandInputPolicy,

    /// What to do with the output of the benchmarked commands
    pub command_output_policies: Vec<CommandOutputPolicy>,

    /// Which time unit to use when displaying results
    pub time_unit: Option<Unit>,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            run_bounds: RunBounds::default(),
            warmup_count: 0,
            min_benchmarking_time: 3.0,
            command_failure_action: CmdFailureAction::RaiseError,
            reference_command: None,
            preparation_command: None,
            conclusion_command: None,
            setup_command: None,
            cleanup_command: None,
            output_style: OutputStyleOption::Full,
            sort_order_speed_comparison: SortOrder::MeanTime,
            sort_order_exports: SortOrder::Command,
            executor_kind: ExecutorKind::default(),
            command_output_policies: vec![CommandOutputPolicy::Null],
            time_unit: None,
            command_input_policy: CommandInputPolicy::Null,
        }
    }
}

impl Options {
    pub fn from_cli_arguments<'a>(matches: &ArgMatches) -> Result<Self, OptionsError<'a>> {
        let mut options = Self::default();
        let param_to_u64 = |param| {
            matches
                .get_one::<String>(param)
                .map(|n| {
                    n.parse::<u64>()
                        .map_err(|e| OptionsError::IntParsingError(param, e))
                })
                .transpose()
        };

        options.warmup_count = param_to_u64("warmup")?.unwrap_or(options.warmup_count);

        let mut min_runs = param_to_u64("min-runs")?;
        let mut max_runs = param_to_u64("max-runs")?;

        if let Some(runs) = param_to_u64("runs")? {
            min_runs = Some(runs);
            max_runs = Some(runs);
        }

        match (min_runs, max_runs) {
            (Some(min), None) => {
                options.run_bounds.min = min;
            }
            (None, Some(max)) => {
                // Since the minimum was not explicit we lower it if max is below the default min.
                options.run_bounds.min = cmp::min(options.run_bounds.min, max);
                options.run_bounds.max = Some(max);
            }
            (Some(min), Some(max)) if min > max => {
                return Err(OptionsError::EmptyRunsRange);
            }
            (Some(min), Some(max)) => {
                options.run_bounds.min = min;
                options.run_bounds.max = Some(max);
            }
            (None, None) => {}
        };

        options.setup_command = matches.get_one::<String>("setup").map(String::from);

        options.reference_command = matches.get_one::<String>("reference").map(String::from);

        options.preparation_command = matches
            .get_many::<String>("prepare")
            .map(|values| values.map(String::from).collect::<Vec<String>>());

        options.conclusion_command = matches
            .get_many::<String>("conclude")
            .map(|values| values.map(String::from).collect::<Vec<String>>());

        options.cleanup_command = matches.get_one::<String>("cleanup").map(String::from);

        options.command_output_policies = if matches.get_flag("show-output") {
            vec![CommandOutputPolicy::Inherit]
        } else if let Some(output_values) = matches.get_many::<String>("output") {
            let mut policies = vec![];
            for value in output_values {
                let policy = match value.as_str() {
                    "null" => CommandOutputPolicy::Null,
                    "pipe" => CommandOutputPolicy::Pipe,
                    "inherit" => CommandOutputPolicy::Inherit,
                    arg => {
                        let path = PathBuf::from(arg);
                        if path.components().count() <= 1 {
                            return Err(OptionsError::UnknownOutputPolicy(arg.to_string()));
                        }
                        CommandOutputPolicy::File(path)
                    }
                };
                policies.push(policy);
            }
            policies
        } else {
            vec![CommandOutputPolicy::Null]
        };

        options.output_style = match matches.get_one::<String>("style").map(|s| s.as_str()) {
            Some("full") => OutputStyleOption::Full,
            Some("basic") => OutputStyleOption::Basic,
            Some("nocolor") => OutputStyleOption::NoColor,
            Some("color") => OutputStyleOption::Color,
            Some("none") => OutputStyleOption::Disabled,
            _ => {
                if options
                    .command_output_policies
                    .iter()
                    .any(|policy| *policy == CommandOutputPolicy::Inherit)
                    || !io::stdout().is_terminal()
                {
                    OutputStyleOption::Basic
                } else if env::var_os("TERM")
                    .map(|t| t == "unknown" || t == "dumb")
                    .unwrap_or(!cfg!(target_os = "windows"))
                    || env::var_os("NO_COLOR")
                        .map(|t| !t.is_empty())
                        .unwrap_or(false)
                {
                    OutputStyleOption::NoColor
                } else {
                    OutputStyleOption::Full
                }
            }
        };

        match options.output_style {
            OutputStyleOption::Basic | OutputStyleOption::NoColor => {
                colored::control::set_override(false)
            }
            OutputStyleOption::Full | OutputStyleOption::Color => {
                colored::control::set_override(true)
            }
            OutputStyleOption::Disabled => {}
        };

        (
            options.sort_order_speed_comparison,
            options.sort_order_exports,
        ) = match matches.get_one::<String>("sort").map(|s| s.as_str()) {
            None | Some("auto") => (SortOrder::MeanTime, SortOrder::Command),
            Some("command") => (SortOrder::Command, SortOrder::Command),
            Some("mean-time") => (SortOrder::MeanTime, SortOrder::MeanTime),
            Some(_) => unreachable!("Unknown sort order"),
        };

        options.executor_kind = if matches.get_flag("no-shell") {
            ExecutorKind::Raw
        } else {
            match (
                matches.get_flag("debug-mode"),
                matches.get_one::<String>("shell"),
            ) {
                (false, Some(shell)) if shell == "default" => ExecutorKind::Shell(Shell::default()),
                (false, Some(shell)) if shell == "none" => ExecutorKind::Raw,
                (false, Some(shell)) => ExecutorKind::Shell(Shell::parse_from_str(shell)?),
                (false, None) => ExecutorKind::Shell(Shell::default()),
                (true, Some(shell)) => ExecutorKind::Mock(Some(shell.into())),
                (true, None) => ExecutorKind::Mock(None),
            }
        };

        if matches.get_flag("ignore-failure") {
            options.command_failure_action = CmdFailureAction::Ignore;
        }

        options.time_unit = match matches.get_one::<String>("time-unit").map(|s| s.as_str()) {
            Some("microsecond") => Some(Unit::MicroSecond),
            Some("millisecond") => Some(Unit::MilliSecond),
            Some("second") => Some(Unit::Second),
            _ => None,
        };

        if let Some(time) = matches.get_one::<String>("min-benchmarking-time") {
            options.min_benchmarking_time = time
                .parse::<f64>()
                .map_err(|e| OptionsError::FloatParsingError("min-benchmarking-time", e))?;
        }

        options.command_input_policy = if let Some(path_str) = matches.get_one::<String>("input") {
            if path_str == "null" {
                CommandInputPolicy::Null
            } else {
                let path = PathBuf::from(path_str);
                if !path.exists() {
                    return Err(OptionsError::StdinDataFileDoesNotExist(
                        path_str.to_string(),
                    ));
                }
                CommandInputPolicy::File(path)
            }
        } else {
            CommandInputPolicy::Null
        };

        Ok(options)
    }

    pub fn validate_against_command_list(&mut self, commands: &Commands) -> Result<()> {
        let has_reference_command = self.reference_command.is_some();
        let num_commands = commands.num_commands(has_reference_command);

        if let Some(preparation_command) = &self.preparation_command {
            ensure!(
                preparation_command.len() <= 1 || num_commands == preparation_command.len(),
                "The '--prepare' option has to be provided just once or N times, where N={num_commands} is the \
                 number of benchmark commands (including a potential reference)."
            );
        }

        if let Some(conclusion_command) = &self.conclusion_command {
            ensure!(
                conclusion_command.len() <= 1 || num_commands == conclusion_command.len(),
                "The '--conclude' option has to be provided just once or N times, where N={num_commands} is the \
                 number of benchmark commands (including a potential reference)."
            );
        }

        if self.command_output_policies.len() == 1 {
            self.command_output_policies =
                vec![self.command_output_policies[0].clone(); num_commands];
        } else {
            ensure!(
                self.command_output_policies.len() == num_commands,
                "The '--output' option has to be provided just once or N times, where N={num_commands} is the \
                 number of benchmark commands (including a potential reference)."
            );
        }

        Ok(())
    }
}

#[test]
fn test_default_shell() {
    let shell = Shell::default();

    let s = format!("{shell}");
    assert_eq!(&s, DEFAULT_SHELL);

    let cmd = shell.command();
    assert_eq!(cmd.get_program(), DEFAULT_SHELL);
}

#[test]
fn test_can_parse_shell_command_line_from_str() {
    let shell = Shell::parse_from_str("shell -x 'aaa bbb'").unwrap();

    let s = format!("{shell}");
    assert_eq!(&s, "shell -x 'aaa bbb'");

    let cmd = shell.command();
    assert_eq!(cmd.get_program().to_string_lossy(), "shell");
    assert_eq!(
        cmd.get_args()
            .map(|a| a.to_string_lossy())
            .collect::<Vec<_>>(),
        vec!["-x", "aaa bbb"]
    );

    // Error cases
    assert!(matches!(
        Shell::parse_from_str("shell 'foo").unwrap_err(),
        OptionsError::ShellParseError(_)
    ));

    assert!(matches!(
        Shell::parse_from_str("").unwrap_err(),
        OptionsError::EmptyShell
    ));

    assert!(matches!(
        Shell::parse_from_str("''").unwrap_err(),
        OptionsError::EmptyShell
    ));
}
