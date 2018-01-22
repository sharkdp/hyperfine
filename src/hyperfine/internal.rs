use std::fmt;

use indicatif::{ProgressBar, ProgressStyle};

/// Type alias for unit of time
pub type Second = f64;

/// Threshold for warning about fast execution time
pub const MIN_EXECUTION_TIME: Second = 5e-3;

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
}

/// A set of options for hyperfine
pub struct HyperfineOptions {
    /// Number of warmup runs
    pub warmup_count: u64,

    /// Minimum number of benchmark runs
    pub min_runs: u64,

    /// Minimum benchmarking time
    pub min_time_sec: Second,

    /// Whether or not to ignore non-zero exit codes
    pub failure_action: CmdFailureAction,

    /// Command to run before each timing run
    pub preparation_command: Option<String>,

    /// What color mode to use for output
    pub output_style: OutputStyleOption,
}

impl Default for HyperfineOptions {
    fn default() -> HyperfineOptions {
        HyperfineOptions {
            warmup_count: 0,
            min_runs: 10,
            min_time_sec: 3.0,
            failure_action: CmdFailureAction::RaiseError,
            preparation_command: None,
            output_style: OutputStyleOption::Full,
        }
    }
}

/// Return a pre-configured progress bar
pub fn get_progress_bar(length: u64, msg: &str) -> ProgressBar {
    let progressbar_style = ProgressStyle::default_spinner()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        .template(" {spinner} {msg:<30} {wide_bar} ETA {eta_precise}");

    let progress_bar = ProgressBar::new(length);
    progress_bar.set_style(progressbar_style.clone());
    progress_bar.enable_steady_tick(80);
    progress_bar.set_message(msg);

    progress_bar
}

/// Possible benchmark warnings
pub enum Warnings {
    FastExecutionTime,
    NonZeroExitCode,
}

impl fmt::Display for Warnings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Warnings::FastExecutionTime => write!(
                f,
                "Command took less than {:.0} ms to complete. Results might be inaccurate.",
                MIN_EXECUTION_TIME * 1e3
            ),
            Warnings::NonZeroExitCode => write!(f, "Ignoring non-zero exit code."),
        }
    }
}

/// A max function for f64's without NaNs
pub fn max(vals: &[f64]) -> f64 {
    *vals.iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
}

/// A min function for f64's without NaNs
pub fn min(vals: &[f64]) -> f64 {
    *vals.iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
}

#[test]
fn test_max() {
    assert_eq!(1.0, max(&[1.0]));
    assert_eq!(-1.0, max(&[-1.0]));
    assert_eq!(-1.0, max(&[-2.0, -1.0]));
    assert_eq!(1.0, max(&[-1.0, 1.0]));
    assert_eq!(1.0, max(&[-1.0, 1.0, 0.0]));
}
