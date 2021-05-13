use crate::hyperfine::error::ParameterScanError;
use crate::hyperfine::types::{Command, NumericType, ParameterValue};
use clap::Values;
use rust_decimal::Decimal;
use std::convert::TryInto;
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

    fn validate(&self) -> Result<(), ParameterScanError> {
        if self.end < self.state {
            return Err(ParameterScanError::EmptyRange);
        }

        if self.step == T::from(0) {
            return Err(ParameterScanError::ZeroStep);
        }

        const MAX_PARAMETERS: usize = 100_000;
        match self.size_hint() {
            (_, Some(size)) if size <= MAX_PARAMETERS => Ok(()),
            _ => Err(ParameterScanError::TooLarge),
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.step == T::from(0) {
            return (usize::MAX, None);
        }

        let steps = (self.end - self.state + T::from(1)) / self.step;
        steps
            .into()
            .try_into()
            .map_or((usize::MAX, None), |u| (u, Some(u)))
    }
}

fn build_parameterized_commands<'a, T: Numeric>(
    param_min: T,
    param_max: T,
    step: T,
    command_names: Vec<&'a str>,
    command_strings: Vec<&'a str>,
    param_name: &'a str,
) -> Result<Vec<Command<'a>>, ParameterScanError> {
    let param_range = RangeStep::new(param_min, param_max, step);
    param_range.validate()?;
    let param_count = param_range.size_hint().1.unwrap();
    let command_name_count = command_names.len();

    // `--command-name` should appear exactly once or same count with parameters.
    if command_name_count > 1 && command_name_count != param_count {
        return Err(ParameterScanError::DifferentCommandNameCountWithParameters(
            command_name_count,
            param_count,
        ));
    }

    let mut i = 0;
    let mut commands = vec![];
    for value in param_range {
        for cmd in &command_strings {
            let name = command_names
                .get(i)
                .or_else(|| command_names.get(0))
                .map(|s| *s);
            commands.push(Command::new_parametrized(
                name,
                cmd,
                vec![(param_name, ParameterValue::Numeric(value.into()))],
            ));
            i += 1;
        }
    }
    Ok(commands)
}

pub fn get_parameterized_commands<'a>(
    command_names: Option<Values<'a>>,
    command_strings: Values<'a>,
    mut vals: clap::Values<'a>,
    step: Option<&str>,
) -> Result<Vec<Command<'a>>, ParameterScanError> {
    let command_names = command_names.map_or(vec![], |names| names.collect::<Vec<&str>>());
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
            command_names,
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
    build_parameterized_commands(
        param_min,
        param_max,
        step,
        command_names,
        command_strings,
        param_name,
    )
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
fn test_range_step_validate() {
    let range_step = RangeStep::new(0, 10, 3);
    assert!(range_step.validate().is_ok());

    let range_step = RangeStep::new(
        Decimal::from(0),
        Decimal::from(1),
        Decimal::from_str("0.1").unwrap(),
    );
    assert!(range_step.validate().is_ok());

    let range_step = RangeStep::new(11, 10, 1);
    assert_eq!(
        format!("{}", range_step.validate().unwrap_err()),
        "Empty parameter range"
    );

    let range_step = RangeStep::new(0, 10, 0);
    assert_eq!(
        format!("{}", range_step.validate().unwrap_err()),
        "Zero is not a valid parameter step"
    );

    let range_step = RangeStep::new(0, 100_001, 1);
    assert_eq!(
        format!("{}", range_step.validate().unwrap_err()),
        "Parameter range is too large"
    );
}

#[test]
fn test_get_parameterized_commands_int() {
    let commands =
        build_parameterized_commands(1i32, 7i32, 3i32, vec![], vec!["echo {val}"], "val").unwrap();
    assert_eq!(commands.len(), 3);
    assert_eq!(commands[2].get_name(), "echo 7");
    assert_eq!(commands[2].get_shell_command(), "echo 7");
}

#[test]
fn test_get_parameterized_commands_decimal() {
    let param_min = Decimal::from_str("0").unwrap();
    let param_max = Decimal::from_str("1").unwrap();
    let step = Decimal::from_str("0.33").unwrap();

    let commands = build_parameterized_commands(
        param_min,
        param_max,
        step,
        vec![],
        vec!["echo {val}"],
        "val",
    )
    .unwrap();
    assert_eq!(commands.len(), 4);
    assert_eq!(commands[3].get_name(), "echo 0.99");
    assert_eq!(commands[3].get_shell_command(), "echo 0.99");
}

#[test]
fn test_get_parameterized_command_names() {
    let commands = build_parameterized_commands(
        1i32,
        3i32,
        1i32,
        vec!["name-{val}"],
        vec!["echo {val}"],
        "val",
    )
    .unwrap();
    assert_eq!(commands.len(), 3);
    let command_names = commands
        .iter()
        .map(|c| c.get_name())
        .collect::<Vec<String>>();
    assert_eq!(command_names, vec!["name-1", "name-2", "name-3"]);
}

#[test]
fn test_get_specified_command_names() {
    let commands = build_parameterized_commands(
        1i32,
        3i32,
        1i32,
        vec!["name-a", "name-b", "name-c"],
        vec!["echo {val}"],
        "val",
    )
    .unwrap();
    assert_eq!(commands.len(), 3);
    let command_names = commands
        .iter()
        .map(|c| c.get_name())
        .collect::<Vec<String>>();
    assert_eq!(command_names, vec!["name-a", "name-b", "name-c"]);
}

#[test]
fn test_different_command_name_count_with_parameters() {
    let result = build_parameterized_commands(
        1i32,
        3i32,
        1i32,
        vec!["name-1", "name-2"],
        vec!["echo {val}"],
        "val",
    );
    assert_eq!(
        format!("{}", result.unwrap_err()),
        "Current --command-name count is 2: expected 3 as parameters"
    );
}
