use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

use crate::hyperfine::types::{BenchmarkResult, OutputStyleOption};
use crate::hyperfine::units::{Scalar, Second};

use std::cmp::Ordering;
use std::iter::Iterator;

/// Threshold for warning about fast execution time
pub const MIN_EXECUTION_TIME: Second = 5e-3;

#[cfg(not(windows))]
const TICK_SETTINGS: (&str, u64) = ("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ", 80);

#[cfg(windows)]
const TICK_SETTINGS: (&str, u64) = (r"+-x| ", 200);

/// Return a pre-configured progress bar
pub fn get_progress_bar(length: u64, msg: &str, option: OutputStyleOption) -> ProgressBar {
    let progressbar_style = match option {
        OutputStyleOption::Basic | OutputStyleOption::Color => ProgressStyle::default_bar(),
        _ => ProgressStyle::default_spinner()
            .tick_chars(TICK_SETTINGS.0)
            .template(" {spinner} {msg:<30} {wide_bar} ETA {eta_precise}"),
    };

    let progress_bar = match option {
        OutputStyleOption::Basic | OutputStyleOption::Color => ProgressBar::hidden(),
        _ => ProgressBar::new(length),
    };
    progress_bar.set_style(progressbar_style);
    progress_bar.enable_steady_tick(TICK_SETTINGS.1);
    progress_bar.set_message(msg.to_owned());

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

#[derive(Debug)]
pub struct BenchmarkResultWithRelativeSpeed<'a> {
    pub result: &'a BenchmarkResult,
    pub relative_speed: Scalar,
    pub relative_speed_stddev: Scalar,
    pub is_fastest: bool,
}

fn compare_mean_time(l: &BenchmarkResult, r: &BenchmarkResult) -> Ordering {
    l.mean.partial_cmp(&r.mean).unwrap_or(Ordering::Equal)
}

pub fn compute_relative_speed (
    results: & [BenchmarkResult],
) -> Option<Vec<BenchmarkResultWithRelativeSpeed>> {
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

pub fn write_benchmark_comparison(results: &[BenchmarkResult]) {
    if results.len() < 2 {
        return;
    }

    if let Some(mut annotated_results) = compute_relative_speed(&results) {
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
    } else {
        eprintln!(
            "{}: The benchmark comparison could not be computed as some benchmark times are zero. \
             This could be caused by background interference during the initial calibration phase \
             of hyperfine, in combination with very fast commands (faster than a few milliseconds). \
             Try to re-run the benchmark on a quiet system. If it does not help, you command is \
             most likely too fast to be accurately benchmarked by hyperfine.",
             "Note".bold().red()
        );
    }
}

#[test]
fn test_max() {
    let assert_float_eq = |a, b| {
        assert!((a - b) < f64::EPSILON);
    };

    assert_float_eq(1.0, max(&[1.0]));
    assert_float_eq(-1.0, max(&[-1.0]));
    assert_float_eq(-1.0, max(&[-2.0, -1.0]));
    assert_float_eq(1.0, max(&[-1.0, 1.0]));
    assert_float_eq(1.0, max(&[-1.0, 1.0, 0.0]));
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

    let annotated_results = compute_relative_speed(&results).unwrap();

    assert_relative_eq!(1.5, annotated_results[0].relative_speed);
    assert_relative_eq!(1.0, annotated_results[1].relative_speed);
    assert_relative_eq!(2.5, annotated_results[2].relative_speed);
}

#[test]
fn test_compute_relative_speed_for_zero_times() {
    let results = vec![create_result("cmd1", 1.0), create_result("cmd2", 0.0)];

    let annotated_results = compute_relative_speed(&results);

    assert!(annotated_results.is_none());
}

pub fn tokenize(values: &str) -> Vec<String> {
    let mut tokens = vec![];
    let mut buf = String::new();

    let mut iter = values.chars();
    while let Some(c) = iter.next() {
        match c {
            '\\' => match iter.next() {
                Some(c2 @ ',') | Some(c2 @ '\\') => {
                    buf.push(c2);
                }
                Some(c2) => {
                    buf.push('\\');
                    buf.push(c2);
                }
                None => buf.push('\\'),
            },
            ',' => {
                tokens.push(buf);
                buf = String::new();
            }
            _ => {
                buf.push(c);
            }
        };
    }

    tokens.push(buf);

    tokens
}

#[test]
fn test_tokenize_single_value() {
    assert_eq!(tokenize(r""), vec![""]);
    assert_eq!(tokenize(r"foo"), vec!["foo"]);
    assert_eq!(tokenize(r" "), vec![" "]);
    assert_eq!(tokenize(r"hello\, world!"), vec!["hello, world!"]);
    assert_eq!(tokenize(r"\,"), vec![","]);
    assert_eq!(tokenize(r"\,\,\,"), vec![",,,"]);
    assert_eq!(tokenize(r"\n"), vec![r"\n"]);
    assert_eq!(tokenize(r"\\"), vec![r"\"]);
    assert_eq!(tokenize(r"\\\,"), vec![r"\,"]);
}

#[test]
fn test_tokenize_multiple_values() {
    assert_eq!(tokenize(r"foo,bar,baz"), vec!["foo", "bar", "baz"]);
    assert_eq!(tokenize(r"hello world,foo"), vec!["hello world", "foo"]);

    assert_eq!(tokenize(r"hello\,world!,baz"), vec!["hello,world!", "baz"]);
}

#[test]
fn test_tokenize_empty_values() {
    assert_eq!(tokenize(r"foo,,bar"), vec!["foo", "", "bar"]);
    assert_eq!(tokenize(r",bar"), vec!["", "bar"]);
    assert_eq!(tokenize(r"bar,"), vec!["bar", ""]);
    assert_eq!(tokenize(r",,"), vec!["", "", ""]);
}
