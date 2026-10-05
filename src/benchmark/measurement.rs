use std::process::ExitStatus;

use crate::outlier_detection::modified_zscores;
use crate::util::statistics::{max, mean, median, min, standard_deviation};
use crate::util::units::Second;

/// Performance measurements and exit status from running a single command
#[derive(Debug, Default, Copy, Clone)]
pub struct Measurement {
    /// Wall clock time
    pub time_wall_clock: Second,

    /// Time spent in user mode
    pub time_user: Second,

    /// Time spent in kernel mode
    pub time_system: Second,

    /// Maximum amount of memory used, in bytes
    pub peak_memory_usage: u64,

    /// The exit status of the process
    pub exit_status: ExitStatus,
}

/// Performance measurements and exit statuses for all runs of a command.
#[derive(Debug, Default, Clone)]
pub struct Measurements {
    pub measurements: Vec<Measurement>,
}

impl Measurements {
    pub fn len(&self) -> usize {
        self.measurements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.measurements.is_empty()
    }

    pub fn push(&mut self, measurement: Measurement) {
        self.measurements.push(measurement);
    }

    pub fn wall_clock_times(&self) -> impl Iterator<Item = Second> + '_ {
        self.measurements.iter().map(|m| m.time_wall_clock)
    }

    /// The average wall clock time.
    pub fn time_wall_clock_mean(&self) -> Second {
        mean(self.wall_clock_times())
    }

    /// The standard deviation of wall clock times, if at least two runs were measured.
    pub fn stddev(&self) -> Option<Second> {
        if self.len() < 2 {
            None
        } else {
            let times: Vec<_> = self.wall_clock_times().collect();
            Some(standard_deviation(&times, self.time_wall_clock_mean()))
        }
    }

    /// The median wall clock time.
    pub fn median(&self) -> Second {
        median(&self.wall_clock_times().collect::<Vec<_>>())
    }

    /// The minimum wall clock time.
    pub fn min(&self) -> Second {
        min(self.wall_clock_times())
    }

    /// The maximum wall clock time.
    pub fn max(&self) -> Second {
        max(self.wall_clock_times())
    }

    /// Compute modified Z-scores for the wall clock times.
    pub fn modified_zscores(&self) -> Vec<f64> {
        modified_zscores(&self.wall_clock_times().collect::<Vec<_>>())
    }

    /// The average user time.
    pub fn time_user_mean(&self) -> Second {
        mean(self.measurements.iter().map(|m| m.time_user))
    }

    /// The average system time.
    pub fn time_system_mean(&self) -> Second {
        mean(self.measurements.iter().map(|m| m.time_system))
    }
}
