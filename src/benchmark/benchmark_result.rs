use std::collections::BTreeMap;

use serde::Serialize;

use crate::benchmark::measurement::Measurements;
use crate::quantity::Time;

/// Parameter value and whether it was used in the command line template
#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct Parameter {
    pub value: String,
    #[serde(skip_serializing)]
    pub is_unused: bool,
}

/// Meta data and performance metrics for a single benchmark
#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct BenchmarkResult {
    /// The expanded command line being benchmarked
    pub command: String,

    /// The display name, when it differs from the command line
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Performance metric measurements and exit codes for each run
    #[serde(flatten)]
    pub measurements: Measurements,

    /// Parameter values for this benchmark
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, Parameter>,
}

impl BenchmarkResult {
    pub fn get_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.command)
    }

    /// The average wall clock time
    pub fn mean_wall_clock_time(&self) -> Time {
        self.measurements.time_wall_clock_mean()
    }

    /// The display name of the command, possibly including a list of
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

        format!("{}{}", self.get_name(), parameters)
    }
}
