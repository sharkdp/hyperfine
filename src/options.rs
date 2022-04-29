use std::process::{Command, Stdio};
use std::{cmp, fmt};

use anyhow::ensure;
use atty::Stream;
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

/// How to handle the output of benchmarked commands
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandOutputPolicy {
    /// Discard all output
    Discard,

    /// Show command output on the terminal
    Forward,
}

impl Default for CommandOutputPolicy {
    fn default() -> Self {
        CommandOutputPolicy::Discard
    }
}

impl CommandOutputPolicy {
    pub fn get_stdout_stderr(&self) -> (Stdio, Stdio) {
        match self {
            CommandOutputPolicy::Discard => (Stdio::null(), Stdio::null()),
            CommandOutputPolicy::Forward => (Stdio::inherit(), Stdio::inherit()),
        }
    }
}

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

    /// Command(s) to run before each timing run
    pub preparation_command: Option<Vec<String>>,

    /// Command(s) to run before each *batch* of timing runs, i.e. before each individual benchmark
    pub setup_command: Option<Vec<String>>,

    /// Command to run after each *batch* of timing runs, i.e. after each individual benchmark
    pub cleanup_command: Option<String>,

    /// What color mode to use for the terminal output
    pub output_style: OutputStyleOption,

    /// Determines how we run commands
    pub executor_kind: ExecutorKind,

    /// What to do with the output of the benchmarked command
    pub command_output_policy: CommandOutputPolicy,

    /// Which time unit to use when displaying resuls
    pub time_unit: Option<Unit>,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            run_bounds: RunBounds::default(),
            warmup_count: 0,
            min_benchmarking_time: 3.0,
            command_failure_action: CmdFailureAction::RaiseError,
            preparation_command: None,
            setup_command: None,
            cleanup_command: None,
            output_style: OutputStyleOption::Full,
            executor_kind: ExecutorKind::default(),
            command_output_policy: CommandOutputPolicy::Discard,
            time_unit: None,
        }
    }
}

impl Options {
    pub fn from_cli_arguments<'a>(matches: &ArgMatches) -> Result<Self, OptionsError<'a>> {
        let mut options = Self::default();
        let param_to_u64 = |param| {
            matches
                .value_of(param)
                .map(|n| {
                    n.parse::<u64>()
                        .map_err(|e| OptionsError::NumericParsingError(param, e))
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

        options.setup_command = matches
            .values_of("setup")
            .map(|values| values.map(String::from).collect::<Vec<String>>());

        options.preparation_command = matches
            .values_of("prepare")
            .map(|values| values.map(String::from).collect::<Vec<String>>());

        options.cleanup_command = matches.value_of("cleanup").map(String::from);

        options.command_output_policy = if matches.is_present("show-output") {
            CommandOutputPolicy::Forward
        } else {
            CommandOutputPolicy::Discard
        };

        options.output_style = match matches.value_of("style") {
            Some("full") => OutputStyleOption::Full,
            Some("basic") => OutputStyleOption::Basic,
            Some("nocolor") => OutputStyleOption::NoColor,
            Some("color") => OutputStyleOption::Color,
            Some("none") => OutputStyleOption::Disabled,
            _ => {
                if options.command_output_policy == CommandOutputPolicy::Discard
                    && atty::is(Stream::Stdout)
                {
                    OutputStyleOption::Full
                } else {
                    OutputStyleOption::Basic
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

        options.executor_kind = if matches.is_present("no-shell") {
            ExecutorKind::Raw
        } else {
            match (matches.is_present("debug-mode"), matches.value_of("shell")) {
                (false, Some(shell)) if shell == "default" => ExecutorKind::Shell(Shell::default()),
                (false, Some(shell)) if shell == "none" => ExecutorKind::Raw,
                (false, Some(shell)) => ExecutorKind::Shell(Shell::parse_from_str(shell)?),
                (false, None) => ExecutorKind::Shell(Shell::default()),
                (true, Some(shell)) => ExecutorKind::Mock(Some(shell.into())),
                (true, None) => ExecutorKind::Mock(None),
            }
        };

        if matches.is_present("ignore-failure") {
            options.command_failure_action = CmdFailureAction::Ignore;
        }

        options.time_unit = match matches.value_of("time-unit") {
            Some("millisecond") => Some(Unit::MilliSecond),
            Some("second") => Some(Unit::Second),
            _ => None,
        };

        Ok(options)
    }

    pub fn validate_against_command_list(&self, commands: &Commands) -> Result<()> {
        if let Some(preparation_command) = &self.preparation_command {
            ensure!(
                preparation_command.len() <= 1
                    || commands.num_commands() == preparation_command.len(),
                "The '--prepare' option has to be provided just once or N times, where N is the \
             number of benchmark commands."
            );
        }

        if let Some(setup_command) = &self.setup_command {
            ensure!(
                setup_command.len() <= 1
                    || commands.num_commands() == setup_command.len(),
                "The '--setup' option has to be provided just once or N times, where N is the \
             number of benchmark commands."
            );
        }

        Ok(())
    }
}

#[test]
fn test_default_shell() {
    let shell = Shell::default();

    let s = format!("{}", shell);
    assert_eq!(&s, DEFAULT_SHELL);

    let cmd = shell.command();
    assert_eq!(cmd.get_program(), DEFAULT_SHELL);
}

#[test]
fn test_can_parse_shell_command_line_from_str() {
    let shell = Shell::parse_from_str("shell -x 'aaa bbb'").unwrap();

    let s = format!("{}", shell);
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
