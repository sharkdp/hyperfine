//! Statistics for non-empty samples of finite measurements.

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

    #[test]
    fn mean_preserves_summation_order() {
        assert_eq!(mean(&[1.0]), 1.0);
        assert_eq!(mean(&[1.0, 2.0, 6.0]), 3.0);
        assert_eq!(mean(&[1e16, 1.0, -1e16]), 0.0);
    }

    #[test]
    fn median_handles_odd_even_and_repeated_values() {
        assert_eq!(median(&[7.0]), 7.0);
        assert_eq!(median(&[6.0, 1.0, 2.0]), 2.0);
        assert_eq!(median(&[6.0, 1.0, 3.0, 2.0]), 2.5);
        assert_eq!(median(&[0.0, 0.0, 0.0, 1.0]), 0.0);
        assert_eq!(median(&[-3.0, -1.0, -2.0]), -2.0);
    }

    #[test]
    fn median_preserves_input_order() {
        let values = [3.0, 1.0, 2.0];
        assert_eq!(median(&values), 2.0);
        assert_eq!(values, [3.0, 1.0, 2.0]);
    }

    #[test]
    #[should_panic(expected = "median requires at least one measurement")]
    fn median_rejects_empty_samples() {
        median(&[]);
    }

    #[test]
    fn standard_deviation_uses_sample_variance() {
        assert_eq!(standard_deviation(&[1.0, 2.0, 3.0], 2.0), 1.0);
        assert_eq!(standard_deviation(&[0.0, 0.0], 0.0), 0.0);
        assert_eq!(standard_deviation(&[7.0, 7.0, 7.0], 7.0), 0.0);
    }

    #[test]
    fn matches_statistical_1_0_floating_point_results() {
        // Recorded from statistical 1.0.0, including small durations and
        // small differences between large values. Compare bits to catch
        // changes that would affect the full precision JSON/CSV exports.
        let cases: &[(&[f64], u64, u64, u64)] = &[
            (
                &[0.09, 0.10, 0.14],
                0x3fbc28f5c28f5c29,
                0x3fb999999999999a,
                0x3f9b17ada62cc3bd,
            ),
            (
                &[0.1, 0.2, 0.3, 0.4],
                0x3fd0000000000000,
                0x3fd0000000000000,
                0x3fc08654a2d4f6da,
            ),
            (
                &[1e-9, 2e-9, 3e-9, 9e-9],
                0x3e301b2b29a4692c,
                0x3e25798ee2308c3a,
                0x3e2edf3cdee63e61,
            ),
            (
                &[1e12 + 0.001, 1e12 + 0.002, 1e12 + 0.003],
                0x426d1a94a2000010,
                0x426d1a94a2000010,
                0x3f510785dd689a29,
            ),
        ];

        for &(values, mean_bits, median_bits, stddev_bits) in cases {
            let mean = mean(values);
            assert_eq!(mean.to_bits(), mean_bits);
            assert_eq!(median(values).to_bits(), median_bits);
            assert_eq!(standard_deviation(values, mean).to_bits(), stddev_bits);
        }
    }

    #[test]
    #[should_panic(expected = "standard deviation requires at least two measurements")]
    fn standard_deviation_rejects_single_measurements() {
        standard_deviation(&[1.0], 1.0);
    }
}
