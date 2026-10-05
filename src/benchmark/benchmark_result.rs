use std::collections::BTreeMap;

use serde::Serialize;

use crate::benchmark::measurement::Measurements;
use crate::quantity::Time;

/// Parameter value
#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct Parameter {
    pub value: String,
}

/// Meta data and performance metrics for a single benchmark
#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct BenchmarkResult {
    /// The command line being benchmarked, after parameter substitution.
    /// For example, `sleep {duration}` with `duration=1` becomes `sleep 1`,
    /// regardless of any custom name.
    pub command: String,

    /// The custom name after parameter substitution, if it differs from `command`.
    /// For example, `--command-name="wait {duration}s"` with `duration=1`
    /// produces `Some("wait 1s")`. Without a custom name, or when the expanded
    /// name equals `command`, this is `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The custom name (or `command`), followed by parameters not used in the
    /// command template, in command-line order. For example, `sleep {duration}`
    /// with `duration=1`, `branch=main`, and the custom name `wait {duration}s`
    /// produces `wait 1s (branch = main)`. Without the custom name, this is
    /// `sleep 1 (branch = main)`. Used in summaries and table exports.
    #[serde(skip_serializing)]
    pub display_name: String,

    /// Performance metric measurements and exit codes for each run
    #[serde(flatten)]
    pub measurements: Measurements,

    /// Parameter values for this benchmark
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, Parameter>,
}

impl BenchmarkResult {
    pub fn get_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.command)
    }

    /// The average wall clock time
    pub fn mean_wall_clock_time(&self) -> Time {
        self.measurements.time_wall_clock_mean()
    }
}
