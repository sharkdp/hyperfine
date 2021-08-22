use serde::*;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

use crate::hyperfine::units::Second;

#[cfg(not(windows))]
pub const DEFAULT_SHELL: &str = "sh";

#[cfg(windows)]
pub const DEFAULT_SHELL: &str = "cmd.exe";

#[derive(Debug, Clone, Serialize, Copy, PartialEq, Eq)]
#[serde(untagged)]
pub enum NumericType {
    Int(i32),
    Decimal(Decimal),
}

impl fmt::Display for NumericType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            NumericType::Int(i) => fmt::Display::fmt(&i, f),
            NumericType::Decimal(i) => fmt::Display::fmt(&i, f),
        }
    }
}

impl From<i32> for NumericType {
    fn from(x: i32) -> NumericType {
        NumericType::Int(x)
    }
}

impl From<Decimal> for NumericType {
    fn from(x: Decimal) -> NumericType {
        NumericType::Decimal(x)
    }
}

impl TryFrom<NumericType> for usize {
    type Error = ();

    fn try_from(numeric: NumericType) -> Result<Self, Self::Error> {
        match numeric {
            NumericType::Int(i) => usize::try_from(i).map_err(|_| ()),
            NumericType::Decimal(d) => match d.to_u64() {
                Some(u) => usize::try_from(u).map_err(|_| ()),
                None => Err(()),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterValue {
    Text(String),
    Numeric(NumericType),
}

impl<'a> ToString for ParameterValue {
    fn to_string(&self) -> String {
        match self {
            ParameterValue::Text(ref value) => value.clone(),
            ParameterValue::Numeric(value) => value.to_string(),
        }
    }
}

/// Set of values that will be exported.
// NOTE: `serde` is used for JSON serialization, but not for CSV serialization due to the
// `parameters` map. Update `src/hyperfine/export/csv.rs` with new fields, as appropriate.
#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct BenchmarkResult {
    /// The command that was run
    pub command: String,

    /// The mean run time
    pub mean: Second,

    /// The standard deviation of all run times
    pub stddev: Second,

    /// The median run time
    pub median: Second,

    /// Time spend in user space
    pub user: Second,

    /// Time spent in system space
    pub system: Second,

    /// Min time measured
    pub min: Second,

    /// Max time measured
    pub max: Second,

    /// All run time measurements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times: Option<Vec<Second>>,

    /// All run exit codes
    pub exit_codes: Vec<Option<i32>>,

    /// Any parameter values used
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, String>,
}

impl BenchmarkResult {
    /// Create a new entry with the given values.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        command: String,
        mean: Second,
        stddev: Second,
        median: Second,
        user: Second,
        system: Second,
        min: Second,
        max: Second,
        times: Vec<Second>,
        exit_codes: Vec<Option<i32>>,
        parameters: BTreeMap<String, String>,
    ) -> Self {
        BenchmarkResult {
            command,
            mean,
            stddev,
            median,
            user,
            system,
            min,
            max,
            times: Some(times),
            exit_codes,
            parameters,
        }
    }
}
