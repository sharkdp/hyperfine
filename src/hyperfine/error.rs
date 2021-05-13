use rust_decimal::Error as DecimalError;
use std::error::Error;
use std::fmt;
use std::num;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum ParameterScanError {
    ParseIntError(num::ParseIntError),
    ParseDecimalError(DecimalError),
    EmptyRange,
    TooLarge,
    ZeroStep,
    StepRequired,
    DifferentCommandNameCountWithParameters(usize, usize),
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
            ParameterScanError::DifferentCommandNameCountWithParameters(real, expected) => {
                write!(
                    f,
                    "Current --command-name count is {}: expected {} as parameters",
                    real, expected
                )
            }
        }
    }
}

impl Error for ParameterScanError {}

#[derive(Debug)]
pub enum OptionsError<'a> {
    RunsBelowTwo,
    EmptyRunsRange,
    TooManyCommandNames(usize),
    DifferentCommandNameCountWithParameters(usize, usize),
    NumericParsingError(&'a str, ParseIntError),
}

impl<'a> fmt::Display for OptionsError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            OptionsError::EmptyRunsRange => write!(f, "Empty runs range"),
            OptionsError::RunsBelowTwo => write!(f, "Number of runs below two"),
            OptionsError::TooManyCommandNames(n) => {
                write!(f, "Too many --command-name options: expected {} at most", n)
            }
            OptionsError::DifferentCommandNameCountWithParameters(real, expected) => {
                write!(
                    f,
                    "Current --command-name count is {}: expected {} as parameters",
                    real, expected
                )
            }
            OptionsError::NumericParsingError(cmd, ref err) => write!(
                f,
                "Could not read numeric argument to '--{cmd}': {err}",
                cmd = cmd,
                err = err
            ),
        }
    }
}

impl<'a> Error for OptionsError<'a> {}
