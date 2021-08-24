use std::cmp::Ordering;

use crate::{benchmark_result::BenchmarkResult, units::Scalar};

#[derive(Debug)]
pub struct BenchmarkResultWithRelativeSpeed<'a> {
    pub result: &'a BenchmarkResult,
    pub relative_speed: Scalar,
    pub relative_speed_stddev: Scalar,
    pub is_fastest: bool,
}

pub fn compare_mean_time(l: &BenchmarkResult, r: &BenchmarkResult) -> Ordering {
    l.mean.partial_cmp(&r.mean).unwrap_or(Ordering::Equal)
}

pub fn compute(results: &[BenchmarkResult]) -> Option<Vec<BenchmarkResultWithRelativeSpeed>> {
    let fastest: &BenchmarkResult = results
        .iter()
        .min_by(|&l, &r| compare_mean_time(l, r))
        .expect("at least one benchmark result");

    if fastest.mean == 0.0 {
        return None;
    }

    Some(
        results
            .iter()
            .map(|result| {
                let ratio = result.mean / fastest.mean;

                // https://en.wikipedia.org/wiki/Propagation_of_uncertainty#Example_formulas
                // Covariance asssumed to be 0, i.e. variables are assumed to be independent
                let ratio_stddev = ratio
                    * ((result.stddev / result.mean).powi(2)
                        + (fastest.stddev / fastest.mean).powi(2))
                    .sqrt();

                BenchmarkResultWithRelativeSpeed {
                    result,
                    relative_speed: ratio,
                    relative_speed_stddev: ratio_stddev,
                    is_fastest: result == fastest,
                }
            })
            .collect(),
    )
}

#[cfg(test)]
fn create_result(name: &str, mean: Scalar) -> BenchmarkResult {
    use std::collections::BTreeMap;

    BenchmarkResult {
        command: name.into(),
        mean,
        stddev: 1.0,
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

    let annotated_results = compute(&results).unwrap();

    assert_relative_eq!(1.5, annotated_results[0].relative_speed);
    assert_relative_eq!(1.0, annotated_results[1].relative_speed);
    assert_relative_eq!(2.5, annotated_results[2].relative_speed);
}

#[test]
fn test_compute_relative_speed_for_zero_times() {
    let results = vec![create_result("cmd1", 1.0), create_result("cmd2", 0.0)];

    let annotated_results = compute(&results);

    assert!(annotated_results.is_none());
}
