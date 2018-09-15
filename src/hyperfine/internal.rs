use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

use hyperfine::types::{BenchmarkResult, OutputStyleOption, Second};

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
            .template("\n {spinner} {msg:<30} {wide_bar} ETA {eta_precise}"),
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

pub fn write_benchmark_comparison(results: &Vec<BenchmarkResult>) {
    if results.len() < 2 {
        return;
    }

    // Show which was faster, maybe expand to table later?
    let mut fastest_item: &BenchmarkResult = &results[0];
    let mut longer_items: Vec<&BenchmarkResult> = Vec::new();

    for run in &results[1..] {
        if let Some(Ordering::Less) = fastest_item.mean.partial_cmp(&run.mean) {
            longer_items.push(run);
        } else {
            longer_items.push(fastest_item);
            fastest_item = run;
        }
    }

    println!("{}", "Summary".bold());
    println!("  '{}' ran", fastest_item.command.cyan());
    longer_items.sort_by(|l, r| l.mean.partial_cmp(&r.mean).unwrap_or(Ordering::Equal));

    for item in longer_items {
        let ratio = item.mean / fastest_item.mean;
        // https://en.wikipedia.org/wiki/Propagation_of_uncertainty#Example_formulas
        // Covariance asssumed to be 0, i.e. variables are assumed to be independent
        let ratio_stddev = ratio
            * ((item.stddev / item.mean).powi(2)
                + (fastest_item.stddev / fastest_item.mean).powi(2))
            .sqrt();
        println!(
            "{} ± {} times faster than '{}'",
            format!("{:8.2}", ratio).bold().green(),
            format!("{:.2}", ratio_stddev).green(),
            &item.command.magenta()
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
