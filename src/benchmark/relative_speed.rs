use std::cmp::Ordering;

use super::benchmark_result::BenchmarkResult;
use crate::util::units::Scalar;

#[derive(Debug)]
pub struct BenchmarkResultWithRelativeSpeed<'a> {
    pub result: &'a BenchmarkResult,
    pub relative_speed: Scalar,
    pub relative_speed_stddev: Option<Scalar>,
    pub is_reference: bool,
    pub is_faster: bool,
}

pub fn compare_mean_time(l: &BenchmarkResult, r: &BenchmarkResult) -> Ordering {
    l.mean.partial_cmp(&r.mean).unwrap_or(Ordering::Equal)
}

pub fn fastest(results: &[BenchmarkResult]) -> &BenchmarkResult {
    results
        .iter()
        .min_by(|&l, &r| compare_mean_time(l, r))
        .expect("at least one benchmark result")
}

pub fn compute<'a>(
    reference: &'a BenchmarkResult,
    results: &'a [BenchmarkResult],
) -> Option<Vec<BenchmarkResultWithRelativeSpeed<'a>>> {
    if reference.mean == 0.0 {
        return None;
    }

    Some(
        results
            .iter()
            .map(|result| {
                let is_reference = result == reference;
                let is_faster = result.mean >= reference.mean;

                let ratio = if is_faster {
                    result.mean / reference.mean
                } else {
                    reference.mean / result.mean
                };

                // https://en.wikipedia.org/wiki/Propagation_of_uncertainty#Example_formulae
                // Covariance asssumed to be 0, i.e. variables are assumed to be independent
                let ratio_stddev =
                    (result.stddev)
                        .zip(reference.stddev)
                        .map(|(result_stddev, fastest_stddev)| {
                            let (a, b) = (
                                (result_stddev / result.mean),
                                (fastest_stddev / reference.mean),
                            );
                            ratio * (a.powi(2) + b.powi(2)).sqrt()
                        });

                BenchmarkResultWithRelativeSpeed {
                    result,
                    relative_speed: ratio,
                    relative_speed_stddev: ratio_stddev,
                    is_reference,
                    is_faster,
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

    let annotated_results = compute(fastest(&results), &results).unwrap();

    assert_relative_eq!(1.5, annotated_results[0].relative_speed);
    assert_relative_eq!(1.0, annotated_results[1].relative_speed);
    assert_relative_eq!(2.5, annotated_results[2].relative_speed);
}

#[test]
fn test_compute_relative_speed_for_zero_times() {
    let results = vec![create_result("cmd1", 1.0), create_result("cmd2", 0.0)];

    let annotated_results = compute(fastest(&results), &results);

    assert!(annotated_results.is_none());
}
