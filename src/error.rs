use std::num::{self, ParseFloatError, ParseIntError};

use rust_decimal::Error as DecimalError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParameterScanError {
    #[error("Error while parsing parameter scan arguments ({0})")]
    ParseIntError(num::ParseIntError),
    #[error("Error while parsing parameter scan arguments ({0})")]
    ParseDecimalError(DecimalError),
    #[error("Empty parameter range")]
    EmptyRange,
    #[error("Parameter range is too large")]
    TooLarge,
    #[error("Zero is not a valid parameter step")]
    ZeroStep,
    #[error("A step size is required when the range bounds are floating point numbers. The step size can be specified with the '-D/--parameter-step-size <DELTA>' parameter")]
    StepRequired,
    #[error("'--command-name' has been specified {0} times. It has to appear exactly once, or exactly {1} times (number of benchmarks)")]
    UnexpectedCommandNameCount(usize, usize),
}

impl From<num::ParseIntError> for ParameterScanError {
    fn from(e: num::ParseIntError) -> ParameterScanError {
        ParameterScanError::ParseIntError(e)
    }
}

impl From<DecimalError> for ParameterScanError {
    fn from(e: DecimalError) -> ParameterScanError {
        ParameterScanError::ParseDecimalError(e)
    }
}

#[derive(Debug, Error)]
pub enum OptionsError<'a> {
    #[error(
        "Conflicting requirements for the number of runs (empty range, min is larger than max)"
    )]
    EmptyRunsRange,
    #[error("Too many --command-name options: Expected {0} at most")]
    TooManyCommandNames(usize),
    #[error("'--command-name' has been specified {0} times. It has to appear exactly once, or exactly {1} times (number of benchmarks)")]
    UnexpectedCommandNameCount(usize, usize),
    #[error("Could not read numeric integer argument to '--{0}': {1}")]
    IntParsingError(&'a str, ParseIntError),
    #[error("Could not read numeric floating point argument to '--{0}': {1}")]
    FloatParsingError(&'a str, ParseFloatError),
    #[error("An empty command has been specified for the '--shell <command>' option")]
    EmptyShell,
    #[error("Failed to parse '--shell <command>' expression as command line: {0}")]
    ShellParseError(shell_words::ParseError),
    #[error("Unknown output policy '{0}'. Use './{0}' to output to a file named '{0}'.")]
    UnknownOutputPolicy(String),
    #[error("The file '{0}' specified as '--input' does not exist")]
    StdinDataFileDoesNotExist(String),
}
