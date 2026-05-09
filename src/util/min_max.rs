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

pub struct Statistics {
    pub min: u64,
    pub max: u64,
    pub mean: u64,
    pub median: u64,
}

/// A function to comupute statistics for u64's
#[must_use]
pub fn statistics(vals: &[u64]) -> Statistics {
    assert!(!vals.is_empty());

    let mut copy = vals.to_vec();
    assert!(!copy.is_empty());
    copy.sort_unstable();

    let len = copy.len();
    // For an even set use the upper middle value, since it is higher than the
    // lower middle value, and higher most of the time means worse, to get an
    // actual data sample as result and not an computed average.
    let median_idx = if len.is_multiple_of(2) {
        len / 2 + 1
    } else {
        len / 2
    };

    Statistics {
        min: *copy.first().expect("non-empty"),
        max: *copy.last().expect("non-empty"),
        mean: copy.iter().sum::<u64>() / len as u64,
        median: copy[median_idx],
    }
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
