use std::fmt;

use indicatif::{ProgressBar, ProgressStyle};

/// Type alias for unit of time
pub type Second = f64;

/// Threshold for warning about fast execution time
pub const MIN_EXECUTION_TIME: Second = 5e-3;

/// A set of options for hyperfine
pub struct HyperfineOptions {
    /// Number of warmup runs
    pub warmup_count: u64,

    /// Minimum number of benchmark runs
    pub min_runs: u64,

    /// Minimum benchmarking time
    pub min_time_sec: Second,

    /// Whether or not to ignore non-zero exit codes
    pub ignore_failure: bool,

    /// Command to run before each timing run
    pub preparation_command: Option<String>,
}

impl Default for HyperfineOptions {
    fn default() -> HyperfineOptions {
        HyperfineOptions {
            warmup_count: 0,
            min_runs: 10,
            min_time_sec: 3.0,
            ignore_failure: false,
            preparation_command: None,
        }
    }
}

/// Return a pre-configured progress bar
pub fn get_progress_bar(length: u64, msg: &str) -> ProgressBar {
    let progressbar_style = ProgressStyle::default_spinner()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        .template(" {spinner} {msg:<30} {wide_bar} ETA {eta_precise}");

    let bar = ProgressBar::new(length);
    bar.set_style(progressbar_style.clone());
    bar.enable_steady_tick(80);
    bar.set_message(msg);

    bar
}

/// Possible benchmark warnings
pub enum Warnings {
    FastExecutionTime,
    NonZeroExitCode,
}

impl fmt::Display for Warnings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Warnings::FastExecutionTime => write!(
                f,
                "Command took less than {:.0} ms to complete. Results might be inaccurate.",
                MIN_EXECUTION_TIME * 1e3
            ),
            &Warnings::NonZeroExitCode => write!(f, "Ignoring non-zero exit code."),
        }
    }
}

/// A max function for f64's without NaNs
pub fn max(vals: &[f64]) -> f64 {
    vals.iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
        .clone()
}

/// A min function for f64's without NaNs
pub fn min(vals: &[f64]) -> f64 {
    vals.iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
        .clone()
}
