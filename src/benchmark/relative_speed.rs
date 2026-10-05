use std::cmp::Ordering;

use super::benchmark_result::BenchmarkResult;
use crate::{
    options::SortOrder,
    quantity::{self, Time, Zero},
};

#[derive(Debug)]
pub struct BenchmarkResultWithRelativeSpeed<'a> {
    pub result: &'a BenchmarkResult,
    /// Runtime divided by the reference runtime; values below one are faster.
    pub relative_speed: f64,
    pub relative_speed_stddev: Option<f64>,
    pub is_reference: bool,
    // Less means faster
    pub relative_ordering: Ordering,
}

pub fn compare_mean_time(l: &BenchmarkResult, r: &BenchmarkResult) -> Ordering {
    l.mean_wall_clock_time()
        .partial_cmp(&r.mean_wall_clock_time())
        .unwrap_or(Ordering::Equal)
}

pub fn fastest_of(results: &[BenchmarkResult]) -> &BenchmarkResult {
    results
        .iter()
        .min_by(|&l, &r| compare_mean_time(l, r))
        .expect("at least one benchmark result")
}

fn compute_relative_speeds<'a>(
    results: &'a [BenchmarkResult],
    reference: &'a BenchmarkResult,
    sort_order: SortOrder,
) -> Vec<BenchmarkResultWithRelativeSpeed<'a>> {
    let mut results: Vec<_> = results
        .iter()
        .map(|result| {
            let is_reference = result == reference;
            let relative_ordering = compare_mean_time(result, reference);

            let reference_mean = reference.mean_wall_clock_time();
            if reference_mean == Time::zero() {
                return BenchmarkResultWithRelativeSpeed {
                    result,
                    relative_speed: if is_reference { 1.0 } else { f64::INFINITY },
                    relative_speed_stddev: None,
                    is_reference,
                    relative_ordering,
                };
            }

            let ratio = result.mean_wall_clock_time() / reference_mean;

            // https://en.wikipedia.org/wiki/Propagation_of_uncertainty#Example_formulas
            // Covariance assumed to be 0, i.e. variables are assumed to be independent
            let ratio_stddev = match (
                result.measurements.stddev(),
                reference.measurements.stddev(),
            ) {
                (Some(result_stddev), Some(reference_stddev)) => Some(
                    ((result_stddev / reference_mean).powi(uom::typenum::P2::new())
                        + (ratio * (reference_stddev / reference_mean))
                            .powi(uom::typenum::P2::new()))
                    .sqrt(),
                ),
                _ => None,
            };

            BenchmarkResultWithRelativeSpeed {
                result,
                relative_speed: ratio.get::<quantity::ratio>(),
                relative_speed_stddev: ratio_stddev.map(|r| r.get::<quantity::ratio>()),
                is_reference,
                relative_ordering,
            }
        })
        .collect();

    match sort_order {
        SortOrder::Command => {}
        SortOrder::MeanTime => {
            results.sort_unstable_by(|r1, r2| compare_mean_time(r1.result, r2.result));
        }
    }

    results
}

pub fn compute_with_check_from_reference<'a>(
    results: &'a [BenchmarkResult],
    reference: &'a BenchmarkResult,
    sort_order: SortOrder,
) -> Option<Vec<BenchmarkResultWithRelativeSpeed<'a>>> {
    if fastest_of(results).mean_wall_clock_time() == Time::zero()
        || reference.mean_wall_clock_time() == Time::zero()
    {
        return None;
    }

    Some(compute_relative_speeds(results, reference, sort_order))
}

pub fn compute_with_check(
    results: &[BenchmarkResult],
    sort_order: SortOrder,
) -> Option<Vec<BenchmarkResultWithRelativeSpeed<'_>>> {
    let fastest = fastest_of(results);

    if fastest.mean_wall_clock_time() == Time::zero() {
        return None;
    }

    Some(compute_relative_speeds(results, fastest, sort_order))
}

/// Compute ratios relative to the given reference, or the fastest result if omitted.
/// A zero reference runtime can produce infinite ratios.
pub fn compute<'a>(
    results: &'a [BenchmarkResult],
    sort_order: SortOrder,
    reference: Option<&'a BenchmarkResult>,
) -> Vec<BenchmarkResultWithRelativeSpeed<'a>> {
    let reference = reference.unwrap_or_else(|| fastest_of(results));

    compute_relative_speeds(results, reference, sort_order)
}

#[cfg(test)]
fn create_result(name: &str, mean: f64) -> BenchmarkResult {
    create_result_from_times(name, &[mean])
}

#[cfg(test)]
fn create_result_from_times(name: &str, times: &[f64]) -> BenchmarkResult {
    use std::collections::BTreeMap;

    use crate::benchmark::measurement::{Measurement, Measurements};
    use crate::quantity::{second, Time};

    BenchmarkResult {
        command: name.into(),
        command_with_unused_parameters: name.into(),
        measurements: Measurements {
            measurements: times
                .iter()
                .map(|&time| Measurement {
                    time_wall_clock: Time::new::<second>(time),
                    time_user: Time::new::<second>(time),
                    ..Default::default()
                })
                .collect(),
        },
        parameters: BTreeMap::new(),
    }
}

#[test]
fn test_compute_relative_speed() {
    use approx::assert_relative_eq;

    let results = vec![
        create_result("cmd1", 3.0),
        create_result("cmd2", 2.0),
        create_result("cmd3", 5.0),
    ];

    let annotated_results = compute_with_check(&results, SortOrder::Command).unwrap();

    assert_relative_eq!(1.5, annotated_results[0].relative_speed);
    assert_relative_eq!(1.0, annotated_results[1].relative_speed);
    assert_relative_eq!(2.5, annotated_results[2].relative_speed);
}

#[test]
fn test_compute_relative_speed_with_reference() {
    use approx::assert_relative_eq;

    let results = vec![
        create_result_from_times("cmd2", &[1.0, 2.0, 3.0]),
        create_result_from_times("cmd3", &[4.0, 5.0, 6.0]),
    ];
    let reference = create_result_from_times("reference", &[3.0, 4.0, 5.0]);

    let annotated_results =
        compute_with_check_from_reference(&results, &reference, SortOrder::Command).unwrap();

    assert_relative_eq!(0.5, annotated_results[0].relative_speed);
    assert_relative_eq!(
        0.2795084971874737,
        annotated_results[0].relative_speed_stddev.unwrap()
    );
    assert_relative_eq!(1.25, annotated_results[1].relative_speed);
}

#[test]
fn test_compute_relative_speed_for_zero_times() {
    let results = vec![create_result("cmd1", 1.0), create_result("cmd2", 0.0)];

    let annotated_results = compute_with_check(&results, SortOrder::Command);

    assert!(annotated_results.is_none());
}

#[test]
fn test_reference_ratios_do_not_depend_on_sort_order() {
    use approx::assert_relative_eq;

    let results = vec![
        create_result("reference", 2.0),
        create_result("slower", 3.0),
        create_result("faster", 1.0),
        create_result("equal", 2.0),
    ];

    for sort_order in [SortOrder::Command, SortOrder::MeanTime] {
        let entries = compute(&results, sort_order, Some(&results[0]));
        for entry in entries {
            assert_relative_eq!(
                entry
                    .result
                    .mean_wall_clock_time()
                    .get::<quantity::second>()
                    / 2.0,
                entry.relative_speed
            );
            assert_eq!(entry.is_reference, entry.result.command == "reference");
        }
    }
}

#[test]
fn test_reference_ratios_without_stddev() {
    let results = vec![
        create_result("reference", 2.0),
        create_result("faster", 1.0),
    ];
    let entries = compute(&results, SortOrder::Command, Some(&results[0]));
    assert_eq!(entries[1].relative_speed, 0.5);
    assert_eq!(entries[1].relative_speed_stddev, None);
}

#[test]
fn test_reference_ratios_with_zero_times() {
    let results = vec![
        create_result_from_times("reference", &[1.0, 2.0, 3.0]),
        create_result_from_times("zero", &[0.0, 0.0, 0.0]),
    ];
    let entries = compute(&results, SortOrder::Command, Some(&results[0]));
    assert_eq!(entries[1].relative_speed, 0.0);
    assert_eq!(entries[1].relative_speed_stddev, Some(0.0));

    let entries = compute(&results, SortOrder::Command, Some(&results[1]));
    assert_eq!(entries[0].relative_speed, f64::INFINITY);
    assert_eq!(entries[0].relative_speed_stddev, None);
    assert_eq!(entries[1].relative_speed, 1.0);

    assert!(compute_with_check_from_reference(&results, &results[0], SortOrder::Command).is_none());
}
