use std::num;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ParameterRangeError {
    ParseIntError(num::ParseIntError),
    EmptyRange,
    TooLarge,
}

impl ParameterRangeError {
    fn __description(&self) -> &str {
        match *self {
            ParameterRangeError::ParseIntError(ref e) => e.description(),
            ParameterRangeError::EmptyRange => "Empty parameter range",
            ParameterRangeError::TooLarge => "Parameter range is too large",
        }
    }
}

impl From<num::ParseIntError> for ParameterRangeError {
    fn from(e: num::ParseIntError) -> ParameterRangeError {
        ParameterRangeError::ParseIntError(e)
    }
}

impl fmt::Display for ParameterRangeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.__description())
    }
}

impl Error for ParameterRangeError {
    fn description(&self) -> &str {
        self.__description()
    }
}
