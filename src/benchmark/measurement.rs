use std::process::ExitStatus;

use serde::Serialize;

use crate::quantity::statistics::{max, mean, median, min, modified_zscores, standard_deviation};
use crate::quantity::{serialize_information, serialize_time, Information, Time};
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

fn serialize_optional_information<S>(
    value: &Option<Information>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(value) => serialize_information(value, serializer),
        None => serializer.serialize_none(),
    }
}

fn serialize_optional_count<S>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    #[derive(Serialize)]
    struct Count {
        value: u64,
    }

    value.map(|value| Count { value }).serialize(serializer)
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct HardwareCounters {
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_count"
    )]
    pub cpu_cycles: Option<u64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_count"
    )]
    pub instructions: Option<u64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_count"
    )]
    pub cache_references: Option<u64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_count"
    )]
    pub cache_misses: Option<u64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_count"
    )]
    pub branch_misses: Option<u64>,
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

    /// Peak resident set size, if available on this platform
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_optional_information"
    )]
    pub memory_peak_resident: Option<Information>,

    #[serde(flatten)]
    pub hardware_counters: HardwareCounters,

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

    pub fn wall_clock_times(&self) -> impl Iterator<Item = Time> + '_ {
        self.measurements.iter().map(|m| m.time_wall_clock)
    }

    /// The average wall clock time
    pub fn time_wall_clock_mean(&self) -> Time {
        mean(self.wall_clock_times())
    }

    /// The standard deviation of all wall clock times. Not available if only one run has been performed
    pub fn stddev(&self) -> Option<Time> {
        let times: Vec<_> = self.wall_clock_times().collect(); // TODO: Avoid collecting

        if times.len() < 2 {
            None
        } else {
            Some(standard_deviation(times))
        }
    }

    /// The median wall clock time
    pub fn median(&self) -> Time {
        median(self.wall_clock_times())
    }

    /// The minimum wall clock time
    pub fn min(&self) -> Time {
        min(self.wall_clock_times())
    }

    /// The maximum wall clock time
    pub fn max(&self) -> Time {
        max(self.wall_clock_times())
    }

    /// Compute modified Z-scores for the wall clock times
    pub fn modified_zscores(&self) -> Vec<f64> {
        modified_zscores(&self.wall_clock_times().collect::<Vec<_>>())
    }

    /// The average user time
    pub fn time_user_mean(&self) -> Time {
        mean(self.measurements.iter().map(|m| m.time_user))
    }

    /// The average system time
    pub fn time_system_mean(&self) -> Time {
        mean(self.measurements.iter().map(|m| m.time_system))
    }
}
