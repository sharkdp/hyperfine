use std::collections::BTreeMap;

use serde::Serialize;

use crate::units::Second;

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

    /// All run exit codes
    pub exit_codes: Vec<Option<i32>>,

    /// Any parameter values used
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, String>,
}

impl BenchmarkResult {
    /// Create a new entry with the given values.
    #[allow(clippy::too_many_arguments)]
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
        exit_codes: Vec<Option<i32>>,
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
            exit_codes,
            parameters,
        }
    }
}
