//! Statistics for non-empty samples of finite measurements.

use std::ops::{Add, AddAssign, Div};

use crate::quantity::{byte, ratio, second, Information, Ratio, Time, Zero};

/// A min function that assumes no NaNs and at least one element
pub fn min<Q: PartialOrd>(values: impl IntoIterator<Item = Q>) -> Q {
    values
        .into_iter()
        .min_by(|a, b| a.partial_cmp(b).expect("No NaN values"))
        .expect("'min' requires at least one element")
}

/// A max function that assumes no NaNs and at least one element
pub fn max<Q: PartialOrd>(values: impl IntoIterator<Item = Q>) -> Q {
    values
        .into_iter()
        .max_by(|a, b| a.partial_cmp(b).expect("No NaN values"))
        .expect("'max' requires at least one element")
}

/// Arithmetic mean, summing measurements in their original order.
pub fn mean<Q, P>(values: impl IntoIterator<Item = Q>) -> Q
where
    Q: AddAssign + Zero + Div<Ratio, Output = P>,
    P: Into<Q>,
{
    let mut sum = Q::zero();
    let mut count = 0usize;
    for value in values {
        sum += value;
        count += 1;
    }

    let count = Ratio::new::<ratio>(count as f64);
    (sum / count).into()
}

/// Median without modifying the input sample.
pub fn median<Q, P>(values: impl IntoIterator<Item = Q>) -> Q
where
    Q: Copy + PartialOrd + Add<Output = Q> + Div<Ratio, Output = P>,
    P: Into<Q>,
{
    let mut values = values.into_iter().collect::<Vec<_>>();
    assert!(
        !values.is_empty(),
        "median requires at least one measurement"
    );
    values.sort_unstable_by(|a, b| a.partial_cmp(b).expect("No NaN measurements"));

    let len = values.len();
    if len % 2 == 0 {
        let mid = len / 2;
        let a = &values[mid - 1];
        let b = &values[mid];
        ((*b + *a) / Ratio::new::<ratio>(2.)).into()
    } else {
        values[len / 2]
    }
}

/// This is just sad, but after several hours trying to figure out how to write
/// a generic `standard_deviation` function using `uom` quantities, I gave up.
pub trait UnsafeRawValue {
    fn unsafe_raw_value(&self) -> f64;
    fn unsafe_from_raw_value(value: f64) -> Self;
}

impl UnsafeRawValue for Time {
    fn unsafe_raw_value(&self) -> f64 {
        self.get::<second>()
    }

    fn unsafe_from_raw_value(value: f64) -> Self {
        Time::new::<second>(value)
    }
}

impl UnsafeRawValue for Information {
    fn unsafe_raw_value(&self) -> f64 {
        self.get::<byte>()
    }

    fn unsafe_from_raw_value(value: f64) -> Self {
        Information::new::<byte>(value)
    }
}

/// Sample standard deviation, using an already computed mean.
pub fn standard_deviation<Q: UnsafeRawValue>(values: &[Q], mean: Q) -> Q {
    let mean = mean.unsafe_raw_value();
    assert!(
        values.len() > 1,
        "standard deviation requires at least two measurements"
    );
    let sum = values
        .iter()
        .map(|value| (value.unsafe_raw_value() - mean) * (value.unsafe_raw_value() - mean))
        .fold(0.0, |sum, deviation| sum + deviation);
    assert!(sum >= 0.0, "invalid sum of squared deviations");

    // Preserve the operation order of statistical 1.0: dividing by n - 1
    // before taking the square root also preserves exported floating-point values.
    Q::unsafe_from_raw_value((sum / (values.len() as f64 - 1.0)).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use uom::si::information::kibibyte;
    use uom::si::time::{microsecond, millisecond};

    #[test]
    fn test_min() {
        assert_eq!(1.0, min([1.0]));
        assert_eq!(-1.0, min([-1.0]));
        assert_eq!(-2.0, min([-2.0, -1.0]));
        assert_eq!(-1.0, min([-1.0, 1.0]));
        assert_eq!(-1.0, min([1.0, -1.0, 0.0]));

        let values = vec![
            Information::new::<kibibyte>(1.0),
            Information::new::<byte>(2.0),
            Information::new::<byte>(3.0),
        ];
        assert_eq!(min(&values).get::<byte>(), 2.0);
    }

    #[test]
    fn test_max() {
        assert_eq!(1.0, max([1.0]));
        assert_eq!(-1.0, max([-1.0]));
        assert_eq!(-1.0, max([-2.0, -1.0]));
        assert_eq!(1.0, max([-1.0, 1.0]));
        assert_eq!(1.0, max([-1.0, 1.0, 0.0]));

        let values = vec![
            Information::new::<byte>(1.0),
            Information::new::<kibibyte>(2.0),
            Information::new::<byte>(3.0),
        ];
        assert_eq!(max(&values).get::<kibibyte>(), 2.0);
    }

    #[test]
    fn test_mean() {
        assert_eq!(1.0, mean([1.0]));
        assert_relative_eq!(2.0, mean([1.0, 3.0]));

        let values = [
            Time::new::<millisecond>(100.0),
            Time::new::<millisecond>(200.0),
            Time::new::<microsecond>(600_000.0),
        ];
        let result = mean(values);
        assert_relative_eq!(result.get::<millisecond>(), 300.0);
    }

    #[test]
    fn test_median() {
        assert_eq!(1.0, median([1.0]));
        assert_relative_eq!(2.0, median([1.0, 3.0]));

        let values = [
            Time::new::<millisecond>(100.0),
            Time::new::<millisecond>(200.0),
            Time::new::<microsecond>(600_000.0),
        ];
        let result = median(values);
        assert_relative_eq!(result.get::<millisecond>(), 200.0);

        let values = [
            Time::new::<millisecond>(100.0),
            Time::new::<millisecond>(200.0),
            Time::new::<microsecond>(300_000.0),
            Time::new::<microsecond>(600_000.0),
        ];
        let result = median(values);
        assert_relative_eq!(result.get::<millisecond>(), 250.0);
    }

    #[test]
    fn test_standard_deviation() {
        let values = [
            Time::new::<millisecond>(100.0),
            Time::new::<millisecond>(200.0),
            Time::new::<microsecond>(300_000.0),
        ];
        let result = standard_deviation(&values, mean(values));
        assert_relative_eq!(result.get::<millisecond>(), 100.0);
    }
}
