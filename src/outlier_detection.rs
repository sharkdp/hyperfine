//! A module for statistical outlier detection.
//!
//! References:
//! - Boris Iglewicz and David Hoaglin (1993), "Volume 16: How to Detect and Handle Outliers",
//!   The ASQC Basic References in Quality Control: Statistical Techniques, Edward F. Mykytka,
//!   Ph.D., Editor.

use statistical::median;

/// Minimum modified Z-score for a datapoint to be an outlier. Here, 1.4826 is a factor that
/// converts the MAD to an estimator for the standard deviation. The second factor is the number
/// of standard deviations.
pub const OUTLIER_THRESHOLD: f64 = 1.4826 * 10.0;

/// Compute modifized Z-scores for a given sample. A (unmodified) Z-score is defined by
/// `(x_i - x_mean)/x_stddev` whereas the modified Z-score is defined by `(x_i - x_median)/MAD`
/// where MAD is the median absolute deviation.
///
/// References:
/// - <https://en.wikipedia.org/wiki/Median_absolute_deviation>
pub fn modified_zscores(xs: &[f64]) -> Vec<f64> {
    assert!(!xs.is_empty());

    // Compute sample median:
    let x_median = median(xs);

    // Compute the absolute deviations from the median:
    let deviations: Vec<f64> = xs.iter().map(|x| (x - x_median).abs()).collect();

    // Compute median absolute deviation:
    let mad = median(&deviations);

    // Handle MAD == 0 case
    let mad = if mad > 0.0 { mad } else { f64::EPSILON };

    // Compute modified Z-scores (x_i - x_median) / MAD
    xs.iter().map(|&x| (x - x_median) / mad).collect()
}

/// Return the number of outliers in a given sample. Outliers are defined as data points with a
/// modified Z-score that is larger than `OUTLIER_THRESHOLD`.
pub fn num_outliers(xs: &[f64]) -> usize {
    count_outliers(xs, OUTLIER_THRESHOLD)
}

/// Return the number of outliers in a given sample for a custom threshold.
pub fn count_outliers(xs: &[f64], threshold: f64) -> usize {
    if xs.is_empty() {
        return 0;
    }

    modified_zscores(xs)
        .iter()
        .filter(|&&s| s.abs() > threshold)
        .count()
}

/// Result of filtering outliers from a sample.
pub struct OutlierFilterResult {
    /// Indices of samples that should be kept (sorted).
    pub keep_indices: Vec<usize>,
    /// Number of discarded samples.
    pub num_discarded: usize,
}

/// Remove statistical outliers from a sample, using the same modified Z-score method as the
/// outlier warnings. At least one sample is always kept.
pub fn filter_outliers(xs: &[f64], threshold: f64) -> OutlierFilterResult {
    if xs.is_empty() {
        return OutlierFilterResult {
            keep_indices: vec![],
            num_discarded: 0,
        };
    }

    if xs.len() == 1 {
        return OutlierFilterResult {
            keep_indices: vec![0],
            num_discarded: 0,
        };
    }

    let scores = modified_zscores(xs);
    let keep_indices: Vec<usize> = scores
        .iter()
        .enumerate()
        .filter(|&(_, score)| score.abs() <= threshold)
        .map(|(index, _)| index)
        .collect();

    if keep_indices.is_empty() {
        let x_median = median(xs);
        let best_index = xs
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (*a - x_median)
                    .abs()
                    .partial_cmp(&(*b - x_median).abs())
                    .unwrap()
            })
            .map(|(index, _)| index)
            .unwrap();

        return OutlierFilterResult {
            keep_indices: vec![best_index],
            num_discarded: xs.len() - 1,
        };
    }

    let num_discarded = xs.len() - keep_indices.len();
    OutlierFilterResult {
        keep_indices,
        num_discarded,
    }
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

#[test]
fn test_filter_outliers() {
    let xs = [0.30, 0.29, 0.31, 0.30, 0.30, 4.0];
    let result = filter_outliers(&xs, OUTLIER_THRESHOLD);
    assert_eq!(result.num_discarded, 1);
    assert_eq!(result.keep_indices.len(), 5);
    assert!(!result.keep_indices.contains(&5));

    let xs = [10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 100.0];
    let result = filter_outliers(&xs, OUTLIER_THRESHOLD);
    assert_eq!(result.num_discarded, 1);
    assert_eq!(result.keep_indices.len(), 7);
}
