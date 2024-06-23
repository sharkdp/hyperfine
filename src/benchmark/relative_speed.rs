use std::cmp::Ordering;

use super::benchmark_result::BenchmarkResult;
use crate::{options::SortOrder, util::units::Scalar};

#[derive(Debug)]
pub struct BenchmarkResultWithRelativeSpeed<'a> {
    pub result: &'a BenchmarkResult,
    pub relative_speed: Scalar,
    pub relative_speed_stddev: Option<Scalar>,
    pub is_reference: bool,
    // Less means faster
    pub relative_ordering: Ordering,
}

pub fn compare_mean_time(l: &BenchmarkResult, r: &BenchmarkResult) -> Ordering {
    l.mean.partial_cmp(&r.mean).unwrap_or(Ordering::Equal)
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

            if result.mean == 0.0 {
                return BenchmarkResultWithRelativeSpeed {
                    result,
                    relative_speed: if is_reference { 1.0 } else { f64::INFINITY },
                    relative_speed_stddev: None,
                    is_reference,
                    relative_ordering,
                };
            }

            let ratio = match relative_ordering {
                Ordering::Less => reference.mean / result.mean,
                Ordering::Equal => 1.0,
                Ordering::Greater => result.mean / reference.mean,
            };

            // https://en.wikipedia.org/wiki/Propagation_of_uncertainty#Example_formulas
            // Covariance asssumed to be 0, i.e. variables are assumed to be independent
            let ratio_stddev = match (result.stddev, reference.stddev) {
                (Some(result_stddev), Some(fastest_stddev)) => Some(
                    ratio
                        * ((result_stddev / result.mean).powi(2)
                            + (fastest_stddev / reference.mean).powi(2))
                        .sqrt(),
                ),
                _ => None,
            };

            BenchmarkResultWithRelativeSpeed {
                result,
                relative_speed: ratio,
                relative_speed_stddev: ratio_stddev,
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
    if fastest_of(results).mean == 0.0 || reference.mean == 0.0 {
        return None;
    }

    Some(compute_relative_speeds(results, reference, sort_order))
}

pub fn compute_with_check(
    results: &[BenchmarkResult],
    sort_order: SortOrder,
) -> Option<Vec<BenchmarkResultWithRelativeSpeed>> {
    let fastest = fastest_of(results);

    if fastest.mean == 0.0 {
        return None;
    }

    Some(compute_relative_speeds(results, fastest, sort_order))
}

/// Same as compute_with_check, potentially resulting in relative speeds of infinity
pub fn compute(
    results: &[BenchmarkResult],
    sort_order: SortOrder,
) -> Vec<BenchmarkResultWithRelativeSpeed> {
    let fastest = fastest_of(results);

    compute_relative_speeds(results, fastest, sort_order)
}

#[cfg(test)]
fn create_result(name: &str, mean: Scalar) -> BenchmarkResult {
    use std::collections::BTreeMap;

    BenchmarkResult {
        command: name.into(),
        command_with_unused_parameters: name.into(),
        mean,
        stddev: Some(1.0),
        median: mean,
        user: mean,
        system: 0.0,
        min: mean,
        max: mean,
        times: None,
        memory_usage_byte: None,
        exit_codes: Vec::new(),
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

    let results = vec![create_result("cmd2", 2.0), create_result("cmd3", 5.0)];
    let reference = create_result("cmd2", 4.0);

    let annotated_results =
        compute_with_check_from_reference(&results, &reference, SortOrder::Command).unwrap();

    assert_relative_eq!(2.0, annotated_results[0].relative_speed);
    assert_relative_eq!(1.25, annotated_results[1].relative_speed);
}

#[test]
fn test_compute_relative_speed_for_zero_times() {
    let results = vec![create_result("cmd1", 1.0), create_result("cmd2", 0.0)];

    let annotated_results = compute_with_check(&results, SortOrder::Command);

    assert!(annotated_results.is_none());
}
