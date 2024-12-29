use std::collections::BTreeMap;

use serde::Serialize;
use statistical::{mean, median, standard_deviation};

use crate::util::{
    min_max::{max, min},
    units::Second,
};

#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct BenchmarkRun {
    /// Wall clock time measurement
    pub wall_clock_time: Second,

    /// Time spent in user mode
    pub user_time: Second,

    /// Time spent in kernel mode
    pub system_time: Second,

    /// Maximum memory usage of the process, in bytes
    pub memory_usage_byte: u64,

    /// Exit codes of the process
    pub exit_code: Option<i32>,
}

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

    /// All run time measurements
    pub runs: Vec<BenchmarkRun>,

    /// Parameter values for this benchmark
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, String>,
}

impl BenchmarkResult {
    fn wall_clock_times(&self) -> Vec<Second> {
        self.runs.iter().map(|run| run.wall_clock_time).collect()
    }

    /// The average run time
    pub fn mean(&self) -> Second {
        mean(&self.wall_clock_times())
    }

    /// The standard deviation of all run times. Not available if only one run has been performed
    pub fn stddev(&self) -> Option<Second> {
        let times = self.wall_clock_times();

        let t_mean = mean(&times);
        if times.len() > 1 {
            Some(standard_deviation(&times, Some(t_mean)))
        } else {
            None
        }
    }

    /// The median run time
    pub fn median(&self) -> Second {
        median(&self.wall_clock_times())
    }

    /// The minimum run time
    pub fn min(&self) -> Second {
        min(&self.wall_clock_times())
    }

    /// The maximum run time
    pub fn max(&self) -> Second {
        max(&self.wall_clock_times())
    }

    pub fn user_mean(&self) -> Second {
        mean(
            &self
                .runs
                .iter()
                .map(|run| run.user_time)
                .collect::<Vec<_>>(),
        )
    }

    pub fn system_mean(&self) -> Second {
        mean(
            &self
                .runs
                .iter()
                .map(|run| run.system_time)
                .collect::<Vec<_>>(),
        )
    }
}
