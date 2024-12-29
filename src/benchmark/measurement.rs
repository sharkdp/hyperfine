use serde::Serialize;

use statistical::{mean, median, standard_deviation};

use crate::util::units::Second;
use crate::{
    outlier_detection::modified_zscores,
    util::min_max::{max, min},
};

/// Performance metric measurements and exit code for a single run
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Measurement {
    /// Elapsed wall clock time (real time)
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
pub struct Measurements {
    pub measurements: Vec<Measurement>,
}

impl Measurements {
    pub fn new(measurements: Vec<Measurement>) -> Self {
        Self { measurements }
    }

    pub fn len(&self) -> usize {
        self.measurements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.measurements.is_empty()
    }

    pub fn push(&mut self, measurement: Measurement) {
        self.measurements.push(measurement);
    }

    pub fn wall_clock_times(&self) -> Vec<Second> {
        self.measurements
            .iter()
            .map(|m| m.wall_clock_time)
            .collect()
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
                .measurements
                .iter()
                .map(|m| m.user_time)
                .collect::<Vec<_>>(),
        )
    }

    /// The average system time
    pub fn system_mean(&self) -> Second {
        mean(
            &self
                .measurements
                .iter()
                .map(|m| m.system_time)
                .collect::<Vec<_>>(),
        )
    }

    pub fn modified_zscores(&self) -> Vec<f64> {
        modified_zscores(&self.wall_clock_times())
    }
}
