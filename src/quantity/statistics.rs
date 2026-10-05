use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;

use uom::num_traits;
use uom::si::f64::Ratio;
use uom::si::ratio::ratio;

use super::{byte, second, Information, Time};

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

/// A mean function that assumes at least one element
pub fn mean<Q, P>(values: impl IntoIterator<Item = Q>) -> Q
where
    Q: AddAssign + num_traits::Zero + Div<Ratio, Output = P>,
    P: Into<Q>,
{
    let mut sum = Q::zero();
    let mut count = 0;
    for value in values {
        sum += value;
        count += 1;
    }

    let count = Ratio::new::<ratio>(count as f64);
    (sum / count).into()
}

pub fn median<Q, P>(values: impl IntoIterator<Item = Q>) -> Q
where
    Q: Copy + PartialOrd + Add<Output = Q> + Div<Ratio, Output = P>,
    P: Into<Q>,
{
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort_by(|a, b| a.partial_cmp(b).expect("No NaN values"));

    let len = values.len();
    if len % 2 == 0 {
        let mid = len / 2;
        let a = &values[mid - 1];
        let b = &values[mid];
        ((*a + *b) / Ratio::new::<ratio>(2.)).into()
    } else {
        values[len / 2]
    }
}

fn standard_deviation_f64(values: impl IntoIterator<Item = f64> + Clone) -> f64 {
    let mean_value = mean(values.clone());

    let mut squared_deviations = 0.;
    let mut n = 0;
    for value in values {
        let deviation = value - mean_value;
        squared_deviations += deviation * deviation;
        n += 1;
    }

    (1. / ((n - 1) as f64) * squared_deviations).sqrt()
}

pub fn standard_deviation<Q: UnsafeRawValue>(values: impl IntoIterator<Item = Q> + Clone) -> Q {
    let values: Vec<_> = values.into_iter().map(|q| q.unsafe_raw_value()).collect();
    let result = standard_deviation_f64(values);
    Q::unsafe_from_raw_value(result)
}

/// Compute modifized Z-scores for a given sample. A (unmodified) Z-score is defined by
/// `(x_i - x_mean)/x_stddev` whereas the modified Z-score is defined by `(x_i - x_median)/MAD`
/// where MAD is the median absolute deviation.
///
/// References:
/// - <https://en.wikipedia.org/wiki/Median_absolute_deviation>
pub fn modified_zscores_f64(xs: &[f64]) -> Vec<f64> {
    assert!(!xs.is_empty());

    // Compute sample median:
    let x_median = median(xs.iter().copied());

    // Compute the absolute deviations from the median:
    let deviations: Vec<f64> = xs.iter().map(|x| (x - x_median).abs()).collect();

    // Compute median absolute deviation:
    let mad = median(deviations.iter().copied());

    // Handle MAD == 0 case
    let mad = if mad > 0.0 { mad } else { f64::EPSILON };

    // Compute modified Z-scores (x_i - x_median) / MAD
    xs.iter().map(|&x| (x - x_median) / mad).collect()
}

pub fn modified_zscores<Q: UnsafeRawValue>(values: &[Q]) -> Vec<f64> {
    let values: Vec<_> = values.iter().map(|q| q.unsafe_raw_value()).collect();
    modified_zscores_f64(&values)
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
        let result = standard_deviation(values);
        assert_relative_eq!(result.get::<millisecond>(), 100.0);
    }
}
