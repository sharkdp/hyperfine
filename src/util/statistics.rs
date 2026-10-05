//! Statistics for non-empty samples of finite measurements.

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
pub fn mean(values: &[f64]) -> f64 {
    values.iter().fold(0.0, |sum, value| sum + value) / values.len() as f64
}

/// Median without modifying the input sample.
pub fn median(values: &[f64]) -> f64 {
    assert!(
        !values.is_empty(),
        "median requires at least one measurement"
    );
    let mut sorted = values.to_vec();
    sorted.sort_unstable_by(|a, b| a.partial_cmp(b).expect("No NaN measurements"));

    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        sorted[mid]
    } else {
        (sorted[mid] + sorted[mid - 1]) / 2.0
    }
}

/// Sample standard deviation, using an already computed mean.
pub fn standard_deviation(values: &[f64], mean: f64) -> f64 {
    assert!(
        values.len() > 1,
        "standard deviation requires at least two measurements"
    );
    let sum = values
        .iter()
        .map(|value| (value - mean) * (value - mean))
        .fold(0.0, |sum, deviation| sum + deviation);
    assert!(sum >= 0.0, "invalid sum of squared deviations");

    // Preserve the operation order of statistical 1.0: dividing by n - 1
    // before taking the square root also preserves exported floating-point values.
    (sum / (values.len() as f64 - 1.0)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_min() {
        assert_eq!(1.0, min([1.0]));
        assert_eq!(-1.0, min([-1.0]));
        assert_eq!(-2.0, min([-2.0, -1.0]));
        assert_eq!(-1.0, min([-1.0, 1.0]));
        assert_eq!(-1.0, min([1.0, -1.0, 0.0]));

        let values = vec![1024.0, 2.0, 3.0];
        assert_eq!(*min(&values), 2.0);
    }

    #[test]
    fn test_max() {
        assert_eq!(1.0, max([1.0]));
        assert_eq!(-1.0, max([-1.0]));
        assert_eq!(-1.0, max([-2.0, -1.0]));
        assert_eq!(1.0, max([-1.0, 1.0]));
        assert_eq!(1.0, max([-1.0, 1.0, 0.0]));

        let values = vec![1.0, 2048.0, 3.0];
        assert_eq!(*max(&values), 2048.0);
    }

    #[test]
    fn test_mean() {
        assert_eq!(1.0, mean(&[1.0]));
        assert_relative_eq!(2.0, mean(&[1.0, 3.0]));

        let values = [0.1, 0.2, 0.6];
        assert_relative_eq!(mean(&values), 0.3);
    }

    #[test]
    fn test_median() {
        assert_eq!(1.0, median(&[1.0]));
        assert_relative_eq!(2.0, median(&[1.0, 3.0]));

        let values = [0.1, 0.2, 0.6];
        assert_relative_eq!(median(&values), 0.2);

        let values = [0.1, 0.2, 0.3, 0.6];
        assert_relative_eq!(median(&values), 0.25);
    }

    #[test]
    fn test_standard_deviation() {
        let values = [0.1, 0.2, 0.3];
        assert_relative_eq!(standard_deviation(&values, mean(&values)), 0.1);
    }
}
