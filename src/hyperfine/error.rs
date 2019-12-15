use rust_decimal::Error as DecimalError;
use std::error::Error;
use std::fmt;
use std::num;

#[derive(Debug)]
pub enum ParameterScanError {
    ParseIntError(num::ParseIntError),
    ParseDecimalError(DecimalError),
    EmptyRange,
    TooLarge,
    ZeroStep,
    StepRequired,
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

impl fmt::Display for ParameterScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ParameterScanError::ParseIntError(ref e) => write!(f, "{}", e),
            ParameterScanError::ParseDecimalError(ref e) => write!(f, "{}", e),
            ParameterScanError::EmptyRange => write!(f, "Empty parameter range"),
            ParameterScanError::TooLarge => write!(f, "Parameter range is too large"),
            ParameterScanError::ZeroStep => write!(f, "Zero is not a valid parameter step"),
            ParameterScanError::StepRequired => write!(
                f,
                "A step size is required when the range bounds are \
                 floating point numbers. The step size can be specified \
                 with the '--parameter-step-size' parameter"
            ),
        }
    }
}

impl Error for ParameterScanError {}

#[derive(Debug)]
pub enum OptionsError {
    RunsBelowTwo,
    EmptyRunsRange,
}

impl fmt::Display for OptionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            OptionsError::EmptyRunsRange => write!(f, "Empty runs range"),
            OptionsError::RunsBelowTwo => write!(f, "Number of runs below two"),
        }
    }
}

impl Error for OptionsError {}
