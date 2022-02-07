use std::process::Command;
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
pub struct Options {
    /// Number of warmup runs
    pub warmup_count: u64,

    /// Number of benchmark runs
    pub runs: Runs,

    /// Minimum benchmarking time
    pub min_time_sec: Second,

    /// Whether or not to ignore non-zero exit codes
    pub failure_action: CmdFailureAction,

    /// Command to run before each batch of timing runs
    pub setup_command: Option<String>,

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

impl Default for Options {
    fn default() -> Options {
        Options {
            names: None,
            warmup_count: 0,
            runs: Runs::default(),
            min_time_sec: 3.0,
            failure_action: CmdFailureAction::RaiseError,
            setup_command: None,
            preparation_command: None,
            cleanup_command: None,
            output_style: OutputStyleOption::Full,
            shell: Shell::default(),
            show_output: false,
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
                options.runs.min = min;
            }
            (None, Some(max)) => {
                // Since the minimum was not explicit we lower it if max is below the default min.
                options.runs.min = cmp::min(options.runs.min, max);
                options.runs.max = Some(max);
            }
            (Some(min), Some(max)) if min > max => {
                return Err(OptionsError::EmptyRunsRange);
            }
            (Some(min), Some(max)) => {
                options.runs.min = min;
                options.runs.max = Some(max);
            }
            (None, None) => {}
        };

        options.setup_command = matches.value_of("setup").map(String::from);

        options.preparation_command = matches
            .values_of("prepare")
            .map(|values| values.map(String::from).collect::<Vec<String>>());

        options.cleanup_command = matches.value_of("cleanup").map(String::from);

        options.show_output = matches.is_present("show-output");

        options.output_style = match matches.value_of("style") {
            Some("full") => OutputStyleOption::Full,
            Some("basic") => OutputStyleOption::Basic,
            Some("nocolor") => OutputStyleOption::NoColor,
            Some("color") => OutputStyleOption::Color,
            Some("none") => OutputStyleOption::Disabled,
            _ => {
                if !options.show_output && atty::is(Stream::Stdout) {
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

        if let Some(shell) = matches.value_of("shell") {
            options.shell = Shell::parse(shell)?;
        }

        if matches.is_present("ignore-failure") {
            options.failure_action = CmdFailureAction::Ignore;
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
                preparation_command.len() <= 1 || commands.len() == preparation_command.len(),
                "The '--prepare' option has to be provided just once or N times, where N is the \
             number of benchmark commands."
            );
        }

        Ok(())
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
    assert_eq!(cmd.get_program().to_string_lossy(), "shell");
    assert_eq!(
        cmd.get_args()
            .map(|a| a.to_string_lossy())
            .collect::<Vec<_>>(),
        vec!["-x", "aaa bbb"]
    );

    // Error cases
    assert!(matches!(
        Shell::parse("shell 'foo").unwrap_err(),
        OptionsError::ShellParseError(_)
    ));

    assert!(matches!(
        Shell::parse("").unwrap_err(),
        OptionsError::EmptyShell
    ));

    assert!(matches!(
        Shell::parse("''").unwrap_err(),
        OptionsError::EmptyShell
    ));
}
