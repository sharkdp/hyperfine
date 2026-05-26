use std::collections::BTreeMap;

use serde::Serialize;

use crate::util::units::Second;

use statistical::{mean, median, standard_deviation};

use crate::util::min_max::{max, min};

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

    /// The average run time
    pub mean: Second,

    /// The standard deviation of all run times. Not available if only one run has been performed
    pub stddev: Option<Second>,

    /// The median run time
    pub median: Second,

    /// Time spent in user mode
    pub user: Second,

    /// Time spent in kernel mode
    pub system: Second,

    /// Minimum of all measured times
    pub min: Second,

    /// Maximum of all measured times
    pub max: Second,

    /// All run time measurements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times: Option<Vec<Second>>,

    /// Maximum memory usage of the process, in bytes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_usage_byte: Option<Vec<u64>>,

    /// Exit codes of all command invocations
    pub exit_codes: Vec<Option<i32>>,

    /// Parameter values for this benchmark
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, String>,
}

/// Combine individual parameter benchmark runs into a single aggregated result.
pub fn merge_parameter_benchmark_results(
    results: Vec<BenchmarkResult>,
    template: &str,
) -> BenchmarkResult {
    let mut times = Vec::new();
    let mut memory_usage_byte = Vec::new();
    let mut exit_codes = Vec::new();
    let mut user_values = Vec::new();
    let mut system_values = Vec::new();

    for result in &results {
        if let Some(result_times) = &result.times {
            times.extend(result_times);
        }
        if let Some(memory) = &result.memory_usage_byte {
            memory_usage_byte.extend(memory);
        }
        exit_codes.extend(result.exit_codes.iter().copied());
        user_values.push(result.user);
        system_values.push(result.system);
    }

    let t_mean = mean(&times);
    let t_stddev = if times.len() > 1 {
        Some(standard_deviation(&times, Some(t_mean)))
    } else {
        None
    };

    BenchmarkResult {
        command: template.to_string(),
        command_with_unused_parameters: format!(
            "{template} (aggregated over {} parameter values)",
            results.len()
        ),
        mean: t_mean,
        stddev: t_stddev,
        median: median(&times),
        user: mean(&user_values),
        system: mean(&system_values),
        min: min(&times),
        max: max(&times),
        times: Some(times),
        memory_usage_byte: if memory_usage_byte.is_empty() {
            None
        } else {
            Some(memory_usage_byte)
        },
        exit_codes,
        parameters: BTreeMap::new(),
    }
}
