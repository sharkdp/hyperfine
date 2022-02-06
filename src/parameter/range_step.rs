use std::convert::TryInto;
use std::ops::{Add, AddAssign, Div, Sub};

use crate::error::ParameterScanError;
use crate::util::number::Number;

pub trait Numeric:
    Add<Output = Self>
    + Sub<Output = Self>
    + Div<Output = Self>
    + AddAssign
    + PartialOrd
    + Copy
    + Clone
    + From<i32>
    + Into<Number>
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
            + Into<Number>,
    > Numeric for T
{
}

#[derive(Debug)]
pub struct RangeStep<T> {
    state: T,
    end: T,
    step: T,
}

impl<T: Numeric> RangeStep<T> {
    pub fn new(start: T, end: T, step: T) -> Result<Self, ParameterScanError> {
        if end < start {
            return Err(ParameterScanError::EmptyRange);
        }

        if step == T::from(0) {
            return Err(ParameterScanError::ZeroStep);
        }

        const MAX_PARAMETERS: usize = 100_000;
        match range_step_size_hint(start, end, step) {
            (_, Some(size)) if size <= MAX_PARAMETERS => Ok(Self {
                state: start,
                end,
                step,
            }),
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
        range_step_size_hint(self.state, self.end, self.step)
    }
}

fn range_step_size_hint<T: Numeric>(start: T, end: T, step: T) -> (usize, Option<usize>) {
    if step == T::from(0) {
        return (usize::MAX, None);
    }

    let steps = (end - start + T::from(1)) / step;
    steps
        .into()
        .try_into()
        .map_or((usize::MAX, None), |u| (u, Some(u)))
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_integer_range() {
        let param_range: Vec<i32> = RangeStep::new(0, 10, 3).unwrap().collect();

        assert_eq!(param_range.len(), 4);
        assert_eq!(param_range[0], 0);
        assert_eq!(param_range[3], 9);
    }

    #[test]
    fn test_decimal_range() {
        let param_min = Decimal::from(0);
        let param_max = Decimal::from(1);
        let step = Decimal::from_str("0.1").unwrap();

        let param_range: Vec<Decimal> = RangeStep::new(param_min, param_max, step)
            .unwrap()
            .collect();

        assert_eq!(param_range.len(), 11);
        assert_eq!(param_range[0], Decimal::from(0));
        assert_eq!(param_range[10], Decimal::from(1));
    }

    #[test]
    fn test_range_step_validate() {
        let result = RangeStep::new(0, 10, 3);
        assert!(result.is_ok());

        let result = RangeStep::new(
            Decimal::from(0),
            Decimal::from(1),
            Decimal::from_str("0.1").unwrap(),
        );
        assert!(result.is_ok());

        let result = RangeStep::new(11, 10, 1);
        assert_eq!(format!("{}", result.unwrap_err()), "Empty parameter range");

        let result = RangeStep::new(0, 10, 0);
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Zero is not a valid parameter step"
        );

        let result = RangeStep::new(0, 100_001, 1);
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "Parameter range is too large"
        );
    }
}
