use crate::hyperfine::error::ParameterScanError;
use crate::hyperfine::types::{Command, NumericType, ParameterValue};
use clap::Values;
use rust_decimal::Decimal;
use std::ops::{Add, AddAssign, Div, Sub};
use std::str::FromStr;

trait Numeric:
    Add<Output = Self>
    + Sub<Output = Self>
    + Div<Output = Self>
    + AddAssign
    + PartialOrd
    + Copy
    + Clone
    + From<i32>
    + Into<NumericType>
{
}
impl<
        T: Add<Output = Self>
            + Sub<Output = Self>
            + Div<Output = Self>
            + AddAssign
            + PartialOrd
            + Copy
            + Clone
            + From<i32>
            + Into<NumericType>,
    > Numeric for T
{
}

struct RangeStep<T> {
    state: T,
    end: T,
    step: T,
}

impl<T: Numeric> RangeStep<T> {
    fn new(start: T, end: T, step: T) -> RangeStep<T> {
        RangeStep {
            state: start,
            end,
            step,
        }
    }
}

impl<T: Numeric> Iterator for RangeStep<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state > self.end {
            return None;
        }
        let return_val = self.state;
        self.state += self.step;

        Some(return_val)
    }
}

fn validate_params<T: Numeric>(start: T, end: T, step: T) -> Result<(), ParameterScanError> {
    if end < start {
        return Err(ParameterScanError::EmptyRange);
    }

    if step == T::from(0) {
        return Err(ParameterScanError::ZeroStep);
    }

    const MAX_PARAMETERS: i32 = 100_000;
    let steps = (end - start + T::from(1)) / step;
    if steps > T::from(MAX_PARAMETERS) {
        return Err(ParameterScanError::TooLarge);
    }

    Ok(())
}

fn build_parameterized_commands<'a, T: Numeric>(
    param_min: T,
    param_max: T,
    step: T,
    command_strings: Vec<&'a str>,
    param_name: &'a str,
) -> Result<Vec<Command<'a>>, ParameterScanError> {
    validate_params(param_min, param_max, step)?;
    let param_range = RangeStep::new(param_min, param_max, step);
    let mut commands = vec![];

    for value in param_range {
        for cmd in &command_strings {
            commands.push(Command::new_parametrized(
                cmd,
                vec![(param_name, ParameterValue::Numeric(value.into()))],
            ));
        }
    }
    Ok(commands)
}

pub fn get_parameterized_commands<'a>(
    command_strings: Values<'a>,
    mut vals: clap::Values<'a>,
    step: Option<&str>,
) -> Result<Vec<Command<'a>>, ParameterScanError> {
    let command_strings = command_strings.collect::<Vec<&str>>();
    let param_name = vals.next().unwrap();
    let param_min = vals.next().unwrap();
    let param_max = vals.next().unwrap();

    // attempt to parse as integers
    if let (Ok(param_min), Ok(param_max), Ok(step)) = (
        param_min.parse::<i32>(),
        param_max.parse::<i32>(),
        step.unwrap_or("1").parse::<i32>(),
    ) {
        return build_parameterized_commands(
            param_min,
            param_max,
            step,
            command_strings,
            param_name,
        );
    }

    // try parsing them as decimals
    let param_min = Decimal::from_str(param_min)?;
    let param_max = Decimal::from_str(param_max)?;

    if step.is_none() {
        return Err(ParameterScanError::StepRequired);
    }

    let step = Decimal::from_str(step.unwrap())?;
    build_parameterized_commands(param_min, param_max, step, command_strings, param_name)
}

#[test]
fn test_integer_range() {
    let param_range: Vec<i32> = RangeStep::new(0, 10, 3).collect();

    assert_eq!(param_range.len(), 4);
    assert_eq!(param_range[0], 0);
    assert_eq!(param_range[3], 9);
}

#[test]
fn test_decimal_range() {
    let param_min = Decimal::from(0);
    let param_max = Decimal::from(1);
    let step = Decimal::from_str("0.1").unwrap();

    let param_range: Vec<Decimal> = RangeStep::new(param_min, param_max, step).collect();

    assert_eq!(param_range.len(), 11);
    assert_eq!(param_range[0], Decimal::from(0));
    assert_eq!(param_range[10], Decimal::from(1));
}

#[test]
fn test_get_parameterized_commands_int() {
    let commands =
        build_parameterized_commands(1i32, 7i32, 3i32, vec!["echo {val}"], "val").unwrap();
    assert_eq!(commands.len(), 3);
    assert_eq!(commands[2].get_shell_command(), "echo 7");
}

#[test]
fn test_get_parameterized_commands_decimal() {
    let param_min = Decimal::from_str("0").unwrap();
    let param_max = Decimal::from_str("1").unwrap();
    let step = Decimal::from_str("0.33").unwrap();

    let commands =
        build_parameterized_commands(param_min, param_max, step, vec!["echo {val}"], "val")
            .unwrap();
    assert_eq!(commands.len(), 4);
    assert_eq!(commands[3].get_shell_command(), "echo 0.99");
}
