/// A max function for f64's without NaNs
pub fn max(vals: &[f64]) -> f64 {
    *vals
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
}

/// A min function for f64's without NaNs
pub fn min(vals: &[f64]) -> f64 {
    *vals
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
}

#[test]
fn test_max() {
    let assert_float_eq = |a: f64, b: f64| {
        assert!((a - b).abs() < f64::EPSILON);
    };

    assert_float_eq(1.0, max(&[1.0]));
    assert_float_eq(-1.0, max(&[-1.0]));
    assert_float_eq(-1.0, max(&[-2.0, -1.0]));
    assert_float_eq(1.0, max(&[-1.0, 1.0]));
    assert_float_eq(1.0, max(&[-1.0, 1.0, 0.0]));
}
