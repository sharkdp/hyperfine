use std::collections::BTreeMap;

use serde::Serialize;

/// Set of values that will be exported.
// NOTE: `serde` is used for JSON serialization, but not for CSV serialization due to the
// `parameters` map. Update `src/hyperfine/export/csv.rs` with new fields, as appropriate.
#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct BenchmarkResult {
    /// The full command line of the program that is being benchmarked
    pub command: String,

    /// The full command line of the program that is being benchmarked, possibly including a list of
    /// parameters that were not used in the command line template.
    #[serde(skip_serializing)]
    pub command_with_unused_parameters: String,

    /// The average run time
    pub mean: f64,

    /// The standard deviation of all run times. Not available if only one run has been performed
    pub stddev: Option<f64>,

    /// The median run time
    pub median: f64,

    /// Time spent in user mode
    pub user: f64,

    /// Time spent in kernel mode
    pub system: f64,

    /// Minimum of all measured times
    pub min: f64,

    /// Maximum of all measured times
    pub max: f64,

    /// All run time measurements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times: Option<Vec<f64>>,

    /// Maximum memory usage of the process, in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_usage_byte: Option<Vec<u64>>,

    /// Exit codes of all command invocations
    pub exit_codes: Vec<Option<i32>>,

    /// Parameter values for this benchmark
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, String>,
}
