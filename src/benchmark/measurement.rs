use std::process::ExitStatus;

use serde::Serialize;

use crate::metric::Metric;
use crate::quantity::statistics::{mean, modified_zscores_f64};
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

    /// Total CPU time (user and kernel mode)
    #[serde(serialize_with = "serialize_time")]
    pub time_cpu: Time,

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

    /// Compute modified Z-scores for a metric validated to be available in every run.
    pub fn modified_zscores(&self, metric: Metric) -> Vec<f64> {
        let values: Vec<_> = self
            .measurements
            .iter()
            .map(|m| metric.value(m).expect("Validated metric is available"))
            .collect();
        modified_zscores_f64(&values)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outlier_detection::OUTLIER_THRESHOLD;
    use crate::quantity::{byte, second};

    #[test]
    fn outliers_follow_the_selected_metric() {
        let measurements = Measurements::new(
            [(1.0, 100.0), (1.0, 1.0), (1.0, 1.0), (100.0, 1.0)]
                .iter()
                .copied()
                .map(|(time, memory)| Measurement {
                    time_wall_clock: Time::new::<second>(time),
                    memory_peak_resident: Some(Information::new::<byte>(memory)),
                    ..Measurement::default()
                })
                .collect(),
        );

        assert!(measurements.modified_zscores(Metric::TimeWallClock)[0] < OUTLIER_THRESHOLD);
        assert!(measurements.modified_zscores(Metric::MemoryPeakResident)[0] > OUTLIER_THRESHOLD);
    }
}
