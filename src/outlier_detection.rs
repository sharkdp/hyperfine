//! A module for statistical outlier detection.
//!
//! References:
//! - Boris Iglewicz and David Hoaglin (1993), "Volume 16: How to Detect and Handle Outliers",
//!   The ASQC Basic References in Quality Control: Statistical Techniques, Edward F. Mykytka,
//!   Ph.D., Editor.

/// Minimum modified Z-score for a datapoint to be an outlier. Here, 1.4826 is a factor that
/// converts the MAD to an estimator for the standard deviation. The second factor is the number
/// of standard deviations.
pub const OUTLIER_THRESHOLD: f64 = 1.4826 * 10.0;

/// Return the number of outliers in a given sample. Outliers are defined as data points with a
/// modified Z-score that is larger than `OUTLIER_THRESHOLD`.
#[cfg(test)]
pub fn num_outliers(xs: &[f64]) -> usize {
    use crate::quantity::statistics::modified_zscores_f64;

    if xs.is_empty() {
        return 0;
    }

    let scores = modified_zscores_f64(xs);
    scores
        .iter()
        .filter(|&&s| s.abs() > OUTLIER_THRESHOLD)
        .count()
}

#[test]
fn test_detect_outliers() {
    // Should not detect outliers in small samples
    assert_eq!(0, num_outliers(&[]));
    assert_eq!(0, num_outliers(&[50.0]));
    assert_eq!(0, num_outliers(&[1000.0, 0.0]));

    // Should not detect outliers in low-variance samples
    let xs = [-0.2, 0.0, 0.2];
    assert_eq!(0, num_outliers(&xs));

    // Should detect a single outlier
    let xs = [-0.2, 0.0, 0.2, 4.0];
    assert_eq!(1, num_outliers(&xs));

    // Should detect a single outlier
    let xs = [0.5, 0.30, 0.29, 0.31, 0.30];
    assert_eq!(1, num_outliers(&xs));

    // Should detect no outliers in sample drawn from normal distribution
    let xs = [
        2.33269488,
        1.42195907,
        -0.57527698,
        -0.31293437,
        2.2948158,
        0.75813273,
        -1.0712388,
        -0.96394741,
        -1.15897446,
        1.10976285,
    ];
    assert_eq!(0, num_outliers(&xs));

    // Should detect two outliers that were manually added
    let xs = [
        2.33269488,
        1.42195907,
        -0.57527698,
        -0.31293437,
        2.2948158,
        0.75813273,
        -1.0712388,
        -0.96394741,
        -1.15897446,
        1.10976285,
        20.0,
        -500.0,
    ];
    assert_eq!(2, num_outliers(&xs));
}

#[test]
fn test_detect_outliers_if_mad_becomes_0() {
    // See https://stats.stackexchange.com/q/339932
    let xs = [10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 100.0];
    assert_eq!(1, num_outliers(&xs));

    let xs = [10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 100.0, 100.0];
    assert_eq!(2, num_outliers(&xs));
}
