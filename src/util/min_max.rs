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

#[test]
fn test_min() {
    let assert_float_eq = |a: f64, b: f64| {
        assert!((a - b).abs() < f64::EPSILON);
    };

    assert_float_eq(1.0, min(&[1.0]));
    assert_float_eq(-1.0, min(&[-1.0]));
    assert_float_eq(-2.0, min(&[-2.0, -1.0]));
    assert_float_eq(-1.0, min(&[-1.0, 1.0]));
    assert_float_eq(-1.0, min(&[-1.0, 1.0, 0.0]));
}

#[test]
#[should_panic]
fn test_max_empty_slice() {
    max(&[]);
}

#[test]
#[should_panic]
fn test_min_empty_slice() {
    min(&[]);
}

#[test]
#[should_panic]
fn test_max_nan() {
    max(&[1.0, f64::NAN]);
}

#[test]
#[should_panic]
fn test_min_nan() {
    min(&[1.0, f64::NAN]);
}
