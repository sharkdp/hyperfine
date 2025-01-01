use std::process::ExitStatus;

use serde::Serialize;

use crate::benchmark::quantity::{max, mean, median, min, standard_deviation, Byte, Second};
use crate::outlier_detection::modified_zscores;
use crate::util::exit_code::extract_exit_code;

fn serialize_exit_status<S>(exit_status: &ExitStatus, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match extract_exit_code(*exit_status) {
        Some(code) => serializer.serialize_i32(code),
        None => serializer.serialize_unit(),
    }
}

/// Performance metric measurements and exit code for a single run
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct Measurement {
    /// Elapsed wall clock time (real time)
    pub time_wall_clock: Second,

    /// Time spent in user mode
    pub time_user: Second,

    /// Time spent in kernel mode
    pub time_system: Second,

    /// Maximum memory usage of the process
    pub peak_memory_usage: Byte,

    // The exit status of the process
    #[serde(rename = "exit_code", serialize_with = "serialize_exit_status")]
    pub exit_status: ExitStatus,
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
            .map(|m| m.time_wall_clock)
            .collect()
    }

    /// The average wall clock time
    pub fn time_wall_clock_mean(&self) -> Second {
        mean(&self.wall_clock_times())
    }

    /// The standard deviation of all wall clock times. Not available if only one run has been performed
    pub fn stddev(&self) -> Option<Second> {
        let times = self.wall_clock_times();

        if times.len() < 2 {
            None
        } else {
            Some(standard_deviation(&times))
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
    pub fn time_user_mean(&self) -> Second {
        mean(
            &self
                .measurements
                .iter()
                .map(|m| m.time_user)
                .collect::<Vec<_>>(),
        )
    }

    /// The average system time
    pub fn time_system_mean(&self) -> Second {
        mean(
            &self
                .measurements
                .iter()
                .map(|m| m.time_system)
                .collect::<Vec<_>>(),
        )
    }

    pub fn peak_memory_usage_mean(&self) -> Byte {
        self.measurements
            .iter()
            .map(|m| m.peak_memory_usage)
            .max_by(|a, b| a.partial_cmp(b).unwrap()) // TODO
            .unwrap() // TODO
    }

    pub fn modified_zscores(&self) -> Vec<f64> {
        modified_zscores(
            &self
                .wall_clock_times()
                .iter()
                .map(|t| t.value_in::<Second>()) // TODO
                .collect::<Vec<_>>(),
        )
    }
}
