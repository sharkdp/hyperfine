use std::cmp::Ordering;

use super::benchmark_result::BenchmarkResult;
use crate::util::units::Scalar;

#[derive(Debug)]
pub struct BenchmarkResultWithRelativeSpeed<'a> {
    pub result: &'a BenchmarkResult,
    pub relative_speed: Scalar,
    pub relative_speed_stddev: Option<Scalar>,
    pub is_fastest: bool,
}

pub fn compare_mean_time(l: &BenchmarkResult, r: &BenchmarkResult) -> Ordering {
    l.mean.partial_cmp(&r.mean).unwrap_or(Ordering::Equal)
}

fn fastest_of(results: &[BenchmarkResult]) -> &BenchmarkResult {
    results
        .iter()
        .min_by(|&l, &r| compare_mean_time(l, r))
        .expect("at least one benchmark result")
}

fn compute_relative_speeds<'a>(
    results: &'a [BenchmarkResult],
    fastest: &'a BenchmarkResult,
) -> Vec<BenchmarkResultWithRelativeSpeed<'a>> {
    results
        .iter()
        .map(|result| {
            let is_fastest = result == fastest;

            if result.mean == 0.0 {
                return BenchmarkResultWithRelativeSpeed {
                    result,
                    relative_speed: if is_fastest { 1.0 } else { f64::INFINITY },
                    relative_speed_stddev: None,
                    is_fastest,
                };
            }

            let ratio = result.mean / fastest.mean;

            // https://en.wikipedia.org/wiki/Propagation_of_uncertainty#Example_formulas
            // Covariance asssumed to be 0, i.e. variables are assumed to be independent
            let ratio_stddev = match (result.stddev, fastest.stddev) {
                (Some(result_stddev), Some(fastest_stddev)) => Some(
                    ratio
                        * ((result_stddev / result.mean).powi(2)
                            + (fastest_stddev / fastest.mean).powi(2))
                        .sqrt(),
                ),
                _ => None,
            };

            BenchmarkResultWithRelativeSpeed {
                result,
                relative_speed: ratio,
                relative_speed_stddev: ratio_stddev,
                is_fastest,
            }
        })
        .collect()
}

pub fn compute_with_check(
    results: &[BenchmarkResult],
) -> Option<Vec<BenchmarkResultWithRelativeSpeed>> {
    let fastest = fastest_of(results);

    if fastest.mean == 0.0 {
        return None;
    }

    Some(compute_relative_speeds(results, fastest))
}

/// Same as compute_with_check, potentially resulting in relative speeds of infinity
pub fn compute(results: &[BenchmarkResult]) -> Vec<BenchmarkResultWithRelativeSpeed> {
    let fastest = fastest_of(results);

    compute_relative_speeds(results, fastest)
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

    let annotated_results = compute_with_check(&results).unwrap();

    assert_relative_eq!(1.5, annotated_results[0].relative_speed);
    assert_relative_eq!(1.0, annotated_results[1].relative_speed);
    assert_relative_eq!(2.5, annotated_results[2].relative_speed);
}

#[test]
fn test_compute_relative_speed_for_zero_times() {
    let results = vec![create_result("cmd1", 1.0), create_result("cmd2", 0.0)];

    let annotated_results = compute_with_check(&results);

    assert!(annotated_results.is_none());
}
