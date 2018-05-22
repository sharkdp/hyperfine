use std::error::Error;
use std::fmt;
use std::num;

#[derive(Debug)]
pub enum ParameterScanError {
    ParseIntError(num::ParseIntError),
    EmptyRange,
    TooLarge,
}

impl ParameterScanError {
    fn __description(&self) -> &str {
        match *self {
            ParameterScanError::ParseIntError(ref e) => e.description(),
            ParameterScanError::EmptyRange => "Empty parameter range",
            ParameterScanError::TooLarge => "Parameter range is too large",
        }
    }
}

impl From<num::ParseIntError> for ParameterScanError {
    fn from(e: num::ParseIntError) -> ParameterScanError {
        ParameterScanError::ParseIntError(e)
    }
}

impl fmt::Display for ParameterScanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.__description())
    }
}

impl Error for ParameterScanError {
    fn description(&self) -> &str {
        self.__description()
    }
}
