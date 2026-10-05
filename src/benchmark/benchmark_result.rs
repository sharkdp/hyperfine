use std::collections::BTreeMap;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::benchmark::measurement::Measurements;
use crate::quantity::{byte, second, Time};
use crate::util::exit_code::extract_exit_code;

/// Parameter value and whether it was used in the command line template
#[derive(Debug, Default, Clone, Serialize, PartialEq)]
#[serde(transparent)]
pub struct Parameter {
    pub value: String,
    #[serde(skip)]
    pub is_unused: bool,
}

/// Meta data and performance metrics for a single benchmark
#[derive(Debug, Default, Clone, PartialEq)]
pub struct BenchmarkResult {
    /// The full command line of the program that is being benchmarked
    pub command: String,

    /// The full command line, including parameters not used in the command template.
    pub command_with_unused_parameters: String,

    /// Performance measurements and exit statuses for each run
    pub measurements: Measurements,

    /// Parameter values for this benchmark
    pub parameters: BTreeMap<String, Parameter>,
}

impl BenchmarkResult {
    /// The average wall clock time
    pub fn mean_wall_clock_time(&self) -> Time {
        self.measurements.time_wall_clock_mean()
    }
}

// Preserve the existing export format while storing typed measurements internally.
impl Serialize for BenchmarkResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct(
            "BenchmarkResult",
            if self.parameters.is_empty() { 11 } else { 12 },
        )?;
        state.serialize_field("command", &self.command)?;
        state.serialize_field("mean", &self.mean_wall_clock_time().get::<second>())?;
        state.serialize_field(
            "stddev",
            &self.measurements.stddev().map(|time| time.get::<second>()),
        )?;
        state.serialize_field("median", &self.measurements.median().get::<second>())?;
        state.serialize_field("user", &self.measurements.time_user_mean().get::<second>())?;
        state.serialize_field(
            "system",
            &self.measurements.time_system_mean().get::<second>(),
        )?;
        state.serialize_field("min", &self.measurements.min().get::<second>())?;
        state.serialize_field("max", &self.measurements.max().get::<second>())?;
        state.serialize_field(
            "times",
            &self
                .measurements
                .wall_clock_times()
                .map(|time| time.get::<second>())
                .collect::<Vec<_>>(),
        )?;
        state.serialize_field(
            "memory_usage_byte",
            &self
                .measurements
                .measurements
                .iter()
                .map(|measurement| measurement.peak_memory_usage.get::<byte>() as u64)
                .collect::<Vec<_>>(),
        )?;
        state.serialize_field(
            "exit_codes",
            &self
                .measurements
                .measurements
                .iter()
                .map(|measurement| extract_exit_code(measurement.exit_status))
                .collect::<Vec<_>>(),
        )?;
        if !self.parameters.is_empty() {
            state.serialize_field("parameters", &self.parameters)?;
        }
        state.end()
    }
}
