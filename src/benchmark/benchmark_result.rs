use std::collections::BTreeMap;

use serde::Serialize;

use crate::util::units::Second;

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
    pub mean: Second,

    /// The standard deviation of all run times. Not available if only one run has been performed
    pub stddev: Option<Second>,

    /// The median run time
    pub median: Second,

    /// Time spent in user mode
    pub user: Second,

    /// Time spent in kernel mode
    pub system: Second,

    /// Minimum of all measured times
    pub min: Second,

    /// Maximum of all measured times
    pub max: Second,

    /// All run time measurements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times: Option<Vec<Second>>,

    /// Maximum memory usage of the process, in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_usage_byte: Option<Vec<u64>>,

    /// Exit codes of all command invocations
    pub exit_codes: Vec<Option<i32>>,

    /// Parameter values for this benchmark
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, String>,
}

impl BenchmarkResult {
    /// Returns true if any run had a non-zero or missing exit code.
    pub fn has_failure(&self) -> bool {
        self.exit_codes.iter().any(|code| *code != Some(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result_with_exit_codes(codes: Vec<Option<i32>>) -> BenchmarkResult {
        BenchmarkResult {
            exit_codes: codes,
            ..Default::default()
        }
    }

    #[test]
    fn test_has_failure_all_success() {
        let r = result_with_exit_codes(vec![Some(0), Some(0), Some(0)]);
        assert!(!r.has_failure());
    }

    #[test]
    fn test_has_failure_with_nonzero() {
        let r = result_with_exit_codes(vec![Some(0), Some(1), Some(0)]);
        assert!(r.has_failure());
    }

    #[test]
    fn test_has_failure_with_signal_kill() {
        let r = result_with_exit_codes(vec![Some(0), None]);
        assert!(r.has_failure());
    }

    #[test]
    fn test_has_failure_empty() {
        let r = result_with_exit_codes(vec![]);
        assert!(!r.has_failure());
    }
}
