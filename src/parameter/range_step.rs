use std::convert::TryInto;
use std::ops::{Add, AddAssign, Div, Rem, Sub};

use crate::error::ParameterScanError;
use crate::util::number::Number;

pub trait Numeric:
    Add<Output = Self>
    + Sub<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
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
            + Rem<Output = Self>
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

        if step < T::from(0) {
            return Err(ParameterScanError::NegativeStep);
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
    if start > end {
        return (0, Some(0));
    }

    if step == T::from(0) {
        return (usize::MAX, None);
    }

    // Remove the remainder first so Decimal division cannot round up a partial
    // step. Count the starting value after converting the whole steps to usize.
    let distance = end - start;
    let steps = (distance - distance % step) / step;
    steps
        .into()
        .try_into()
        .and_then(|u: usize| u.checked_add(1).ok_or(()))
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
    fn size_hint_counts_inclusive_integer_values() {
        let mut range = RangeStep::new(0, 10, 3).unwrap();
        for remaining in (1..=4).rev() {
            assert_eq!(range.size_hint(), (remaining, Some(remaining)));
            assert!(range.next().is_some());
        }
        assert_eq!(range.size_hint(), (0, Some(0)));
        assert_eq!(range.next(), None);
    }

    #[test]
    fn decimal_singleton_with_small_step_is_allowed() {
        let range = RangeStep::new(
            Decimal::ZERO,
            Decimal::ZERO,
            Decimal::from_str("0.000001").unwrap(),
        )
        .unwrap();
        assert_eq!(range.size_hint(), (1, Some(1)));
        assert_eq!(range.collect::<Vec<_>>(), vec![Decimal::ZERO]);
    }

    #[test]
    fn decimal_size_hint_counts_values_not_unit_intervals() {
        let mut range = RangeStep::new(
            Decimal::ZERO,
            Decimal::ONE,
            Decimal::from_str("0.3").unwrap(),
        )
        .unwrap();
        for remaining in (1..=4).rev() {
            assert_eq!(range.size_hint(), (remaining, Some(remaining)));
            assert!(range.next().is_some());
        }
        assert_eq!(range.size_hint(), (0, Some(0)));
        assert_eq!(range.next(), None);
    }

    #[test]
    fn parameter_limit_counts_inclusive_endpoint() {
        let range = RangeStep::new(0, 299_997, 3).unwrap();
        assert_eq!(range.size_hint(), (100_000, Some(100_000)));
        assert_eq!(range.count(), 100_000);
    }

    #[test]
    fn parameter_limit_rejects_one_too_many_values() {
        assert!(matches!(
            RangeStep::new(0, 300_000, 3),
            Err(ParameterScanError::TooLarge)
        ));
    }

    #[test]
    fn decimal_parameter_limit_counts_inclusive_endpoint() {
        let step = Decimal::from_str("0.000001").unwrap();
        let range =
            RangeStep::new(Decimal::ZERO, Decimal::from_str("0.099999").unwrap(), step).unwrap();
        assert_eq!(range.size_hint(), (100_000, Some(100_000)));
        assert_eq!(range.count(), 100_000);
        assert!(matches!(
            RangeStep::new(Decimal::ZERO, Decimal::from_str("0.1").unwrap(), step),
            Err(ParameterScanError::TooLarge)
        ));
    }

    #[test]
    fn decimal_size_hint_does_not_round_up_a_partial_step() {
        let range = RangeStep::new(
            Decimal::ZERO,
            Decimal::from_str("29999999999999999999999999999").unwrap(),
            Decimal::from_str("30000000000000000000000000000").unwrap(),
        )
        .unwrap();
        assert_eq!(range.size_hint(), (1, Some(1)));
        assert_eq!(range.count(), 1);
    }

    #[test]
    fn decimal_parameter_limit_does_not_round_up_a_partial_step() {
        let range = RangeStep::new(
            Decimal::ZERO,
            Decimal::from_str("29999999999999999999999999999").unwrap(),
            Decimal::from_str("300000000000000000000000").unwrap(),
        )
        .unwrap();
        assert_eq!(range.size_hint(), (100_000, Some(100_000)));
        assert_eq!(range.count(), 100_000);
    }

    #[test]
    fn test_negative_integer_step() {
        for step in [-2, -3, i32::MIN, -1] {
            for end in [0, 1, 10] {
                let result = RangeStep::new(0, end, step);
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "Parameter step size must be positive"
                );
            }
        }
    }

    #[test]
    fn test_negative_decimal_step() {
        for step in ["-2.0", "-0.1"] {
            for end in [0, 1, 10] {
                let result = RangeStep::new(
                    Decimal::ZERO,
                    Decimal::from(end),
                    Decimal::from_str(step).unwrap(),
                );
                assert_eq!(
                    result.unwrap_err().to_string(),
                    "Parameter step size must be positive"
                );
            }
        }
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
