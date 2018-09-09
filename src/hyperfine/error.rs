use std::error::Error;
use std::fmt;
use std::num;

#[derive(Debug)]
pub enum ParameterScanError {
    ParseIntError(num::ParseIntError),
    EmptyRange,
    TooLarge,
}

impl From<num::ParseIntError> for ParameterScanError {
    fn from(e: num::ParseIntError) -> ParameterScanError {
        ParameterScanError::ParseIntError(e)
    }
}

impl fmt::Display for ParameterScanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for ParameterScanError {
    fn description(&self) -> &str {
        match *self {
            ParameterScanError::ParseIntError(ref e) => e.description(),
            ParameterScanError::EmptyRange => "Empty parameter range",
            ParameterScanError::TooLarge => "Parameter range is too large",
        }
    }
}

#[derive(Debug)]
pub enum OptionsError {
    RunsBelowTwo,
    EmptyRunsRange,
}

impl fmt::Display for OptionsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for OptionsError {
    fn description(&self) -> &str {
        match *self {
            OptionsError::EmptyRunsRange => "Empty runs range",
            OptionsError::RunsBelowTwo => "Number of runs below two",
        }
    }
}
