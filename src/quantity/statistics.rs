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

/// A max function that assumes no NaNs and at least one element
pub fn max<Q: PartialOrd>(values: impl IntoIterator<Item = Q>) -> Q {
    values
        .into_iter()
        .max_by(|a, b| a.partial_cmp(b).expect("No NaN values"))
        .expect("'max' requires at least one element")
}

/// A min function that assumes no NaNs and at least one element
pub fn min<Q: PartialOrd>(values: impl IntoIterator<Item = Q>) -> Q {
    values
        .into_iter()
        .min_by(|a, b| a.partial_cmp(b).expect("No NaN values"))
        .expect("'min' requires at least one element")
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

pub fn standard_deviation<Q: UnsafeRawValue>(values: &[Q]) -> Q {
    let values: Vec<_> = values.iter().map(|q| q.unsafe_raw_value()).collect();
    let result_value = {
        let mean_value = statistical::mean(&values);
        statistical::standard_deviation(&values, Some(mean_value))
    };
    Q::unsafe_from_raw_value(result_value)
}

pub fn modified_zscores<Q: UnsafeRawValue>(values: &[Q]) -> Vec<f64> {
    let values: Vec<_> = values.iter().map(|q| q.unsafe_raw_value()).collect();
    crate::outlier_detection::modified_zscores(&values)
}

#[test]
fn test_max() {
    assert_eq!(1.0, max([1.0]));
    assert_eq!(-1.0, max([-1.0]));
    assert_eq!(-1.0, max([-2.0, -1.0]));
    assert_eq!(1.0, max([-1.0, 1.0]));
    assert_eq!(1.0, max([-1.0, 1.0, 0.0]));
}

#[test]
fn test_mean() {
    use crate::quantity::{FormatQuantity, TimeUnit};
    use uom::si::time::millisecond;

    let values = vec![
        Time::new::<millisecond>(123.4),
        Time::new::<millisecond>(234.5),
    ];
    let result = mean(values.into_iter());
    assert_eq!(result.format(TimeUnit::MilliSecond), "178.9 ms");
}

#[test]
fn statistics() {
    use crate::quantity::{FormatQuantity, InformationUnit, TimeUnit};

    let values = vec![
        Time::new::<second>(1.0),
        Time::new::<second>(2.0),
        Time::new::<second>(3.0),
    ];

    assert_eq!(
        mean(values.iter().copied()).format(TimeUnit::Second),
        "2.000 s"
    );
    assert_eq!(
        median(values.iter().copied()).format(TimeUnit::Second),
        "2.000 s"
    );
    assert_eq!(min(&values).format(TimeUnit::Second), "1.000 s");
    assert_eq!(max(&values).format(TimeUnit::Second), "3.000 s");
    assert_eq!(
        standard_deviation(&values).format(TimeUnit::Second),
        "1.000 s"
    );

    let values = vec![
        Information::new::<byte>(1.0),
        Information::new::<byte>(2.0),
        Information::new::<byte>(3.0),
    ];
    mean(values.iter().copied()).format(InformationUnit::Byte);
}
