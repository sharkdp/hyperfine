use std::collections::BTreeMap;

use serde::Serialize;
use statistical::{mean, median, standard_deviation};

use crate::{
    outlier_detection::modified_zscores,
    util::{
        min_max::{max, min},
        units::Second,
    },
};

/// Performance metrics and exit codes for each run
#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct Run {
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

#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct Runs {
    pub runs: Vec<Run>,
}

impl Runs {
    pub fn new(runs: Vec<Run>) -> Self {
        Self { runs }
    }

    pub fn len(&self) -> usize {
        self.runs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }

    pub fn push(&mut self, run: Run) {
        self.runs.push(run);
    }

    pub fn wall_clock_times(&self) -> Vec<Second> {
        self.runs.iter().map(|run| run.wall_clock_time).collect()
    }

    /// The average wall clock time
    pub fn mean(&self) -> Second {
        mean(&self.wall_clock_times())
    }

    /// The standard deviation of all wall clock times. Not available if only one run has been performed
    pub fn stddev(&self) -> Option<Second> {
        let times = self.wall_clock_times();

        let t_mean = mean(&times);
        if times.len() > 1 {
            Some(standard_deviation(&times, Some(t_mean)))
        } else {
            None
        }
    }

    /// The median wall clock time
    pub fn median(&self) -> Second {
        median(&self.wall_clock_times())
    }

    /// The minimum wall clock time
    pub fn min(&self) -> Second {
        min(&self.wall_clock_times())
    }

    /// The maximum wall clock time
    pub fn max(&self) -> Second {
        max(&self.wall_clock_times())
    }

    /// The average user time
    pub fn user_mean(&self) -> Second {
        mean(
            &self
                .runs
                .iter()
                .map(|run| run.user_time)
                .collect::<Vec<_>>(),
        )
    }

    /// The average system time
    pub fn system_mean(&self) -> Second {
        mean(
            &self
                .runs
                .iter()
                .map(|run| run.system_time)
                .collect::<Vec<_>>(),
        )
    }

    pub fn modified_zscores(&self) -> Vec<f64> {
        modified_zscores(&self.wall_clock_times())
    }
}

/// Parameter value and whether it was used in the command line template
#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct Parameter {
    pub value: String,
    pub is_unused: bool,
}

/// Meta data and performance metrics for a single benchmark
#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct BenchmarkResult {
    /// The full command line of the program that is being benchmarked
    pub command: String,

    /// Performance metrics and exit codes for each run
    #[serde(flatten)]
    pub runs: Runs,

    /// Parameter values for this benchmark
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, Parameter>,
}

impl BenchmarkResult {
    /// The average wall clock time
    pub fn mean_wall_clock_time(&self) -> Second {
        self.runs.mean()
    }

    /// The full command line of the program that is being benchmarked, possibly including a list of
    /// parameters that were not used in the command line template.
    pub fn command_with_unused_parameters(&self) -> String {
        let parameters = self
            .parameters
            .iter()
            .filter(|(_, parameter)| parameter.is_unused)
            .fold(String::new(), |output, (name, parameter)| {
                output + &format!("{name} = {value}, ", value = parameter.value)
            });
        let parameters = parameters.trim_end_matches(", ");
        let parameters = if parameters.is_empty() {
            "".into()
        } else {
            format!(" ({parameters})")
        };

        format!("{}{}", self.command, parameters)
    }
}
