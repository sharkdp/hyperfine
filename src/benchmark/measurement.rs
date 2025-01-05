use std::process::ExitStatus;

use serde::Serialize;

use crate::outlier_detection::modified_zscores;
use crate::quantity::{
    max, mean, median, min, second, serialize_information, serialize_time, standard_deviation,
    Information, Time, TimeQuantity,
};
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
    #[serde(serialize_with = "serialize_time")]
    pub time_wall_clock: Time,

    /// Time spent in user mode
    #[serde(serialize_with = "serialize_time")]
    pub time_user: Time,

    /// Time spent in kernel mode
    #[serde(serialize_with = "serialize_time")]
    pub time_system: Time,

    /// Maximum memory usage of the process
    #[serde(serialize_with = "serialize_information")]
    pub peak_memory_usage: Information,

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

    pub fn wall_clock_times(&self) -> Vec<Time> {
        self.measurements
            .iter()
            .map(|m| m.time_wall_clock)
            .collect()
    }

    /// The average wall clock time
    pub fn time_wall_clock_mean(&self) -> Time {
        mean(&self.wall_clock_times())
    }

    /// The standard deviation of all wall clock times. Not available if only one run has been performed
    pub fn stddev(&self) -> Option<Time> {
        let times = self.wall_clock_times();

        if times.len() < 2 {
            None
        } else {
            Some(standard_deviation(&times))
        }
    }

    /// The median wall clock time
    pub fn median(&self) -> Time {
        median(&self.wall_clock_times())
    }

    /// The minimum wall clock time
    pub fn min(&self) -> Time {
        min(&self.wall_clock_times())
    }

    /// The maximum wall clock time
    pub fn max(&self) -> Time {
        max(&self.wall_clock_times())
    }

    /// The average user time
    pub fn time_user_mean(&self) -> Time {
        mean(
            &self
                .measurements
                .iter()
                .map(|m| m.time_user)
                .collect::<Vec<_>>(),
        )
    }

    /// The average system time
    pub fn time_system_mean(&self) -> Time {
        mean(
            &self
                .measurements
                .iter()
                .map(|m| m.time_system)
                .collect::<Vec<_>>(),
        )
    }

    pub fn peak_memory_usage_mean(&self) -> Information {
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
                .map(|t| t.value_in(second)) // TODO
                .collect::<Vec<_>>(),
        )
    }
}
