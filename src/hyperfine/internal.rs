use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

use crate::hyperfine::types::{BenchmarkResult, OutputStyleOption};
use crate::hyperfine::units::{Scalar, Second};

use std::cmp::Ordering;
use std::iter::Iterator;

/// Threshold for warning about fast execution time
pub const MIN_EXECUTION_TIME: Second = 5e-3;

/// Return a pre-configured progress bar
pub fn get_progress_bar(length: u64, msg: &str, option: &OutputStyleOption) -> ProgressBar {
    let progressbar_style = match *option {
        OutputStyleOption::Basic | OutputStyleOption::Color => ProgressStyle::default_bar(),
        _ => ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template(" {spinner} {msg:<30} {wide_bar} ETA {eta_precise}"),
    };

    let progress_bar = match *option {
        OutputStyleOption::Basic | OutputStyleOption::Color => ProgressBar::hidden(),
        _ => ProgressBar::new(length),
    };
    progress_bar.set_style(progressbar_style.clone());
    progress_bar.enable_steady_tick(80);
    progress_bar.set_message(msg);

    progress_bar
}

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

pub struct BenchmarkResultWithRelativeSpeed<'a> {
    pub result: &'a BenchmarkResult,
    pub relative_speed: Scalar,
    pub relative_speed_stddev: Scalar,
}

fn compare_mean_time(l: &BenchmarkResult, r: &BenchmarkResult) -> Ordering {
    l.mean.partial_cmp(&r.mean).unwrap_or(Ordering::Equal)
}

pub fn compute_relative_speed<'a>(
    results: &'a Vec<BenchmarkResult>,
) -> Vec<BenchmarkResultWithRelativeSpeed<'a>> {
    let fastest: &BenchmarkResult = results
        .iter()
        .min_by(|&l, &r| compare_mean_time(l, r))
        .expect("at least one benchmark result");

    results
        .iter()
        .map(|result| {
            let ratio = result.mean / fastest.mean;

            // https://en.wikipedia.org/wiki/Propagation_of_uncertainty#Example_formulas
            // Covariance asssumed to be 0, i.e. variables are assumed to be independent
            let ratio_stddev = ratio
                * ((result.stddev / result.mean).powi(2) + (fastest.stddev / fastest.mean).powi(2))
                    .sqrt();

            BenchmarkResultWithRelativeSpeed {
                result,
                relative_speed: ratio,
                relative_speed_stddev: ratio_stddev,
            }
        })
        .collect()
}

pub fn write_benchmark_comparison(results: &Vec<BenchmarkResult>) {
    if results.len() < 2 {
        return;
    }

    let mut annotated_results = compute_relative_speed(&results);
    annotated_results.sort_by(|l, r| compare_mean_time(l.result, r.result));

    let fastest = &annotated_results[0];
    let others = &annotated_results[1..];

    println!("{}", "Summary".bold());
    println!("  '{}' ran", fastest.result.command.cyan());

    for item in others {
        println!(
            "{} ± {} times faster than '{}'",
            format!("{:8.2}", item.relative_speed).bold().green(),
            format!("{:.2}", item.relative_speed_stddev).green(),
            &item.result.command.magenta()
        );
    }
}

#[test]
fn test_max() {
    assert_eq!(1.0, max(&[1.0]));
    assert_eq!(-1.0, max(&[-1.0]));
    assert_eq!(-1.0, max(&[-2.0, -1.0]));
    assert_eq!(1.0, max(&[-1.0, 1.0]));
    assert_eq!(1.0, max(&[-1.0, 1.0, 0.0]));
}

#[test]
fn test_compute_relative_speed() {
    use approx::assert_relative_eq;

    let create_result = |name: &str, mean| BenchmarkResult {
        command: name.into(),
        mean: mean,
        stddev: 1.0,
        user: mean,
        system: 0.0,
        min: mean,
        max: mean,
        times: None,
        parameter: None,
    };

    let results = vec![
        create_result("cmd1", 3.0),
        create_result("cmd2", 2.0),
        create_result("cmd3", 5.0),
    ];

    let annotated_results = compute_relative_speed(&results);

    assert_relative_eq!(1.5, annotated_results[0].relative_speed);
    assert_relative_eq!(1.0, annotated_results[1].relative_speed);
    assert_relative_eq!(2.5, annotated_results[2].relative_speed);
}
