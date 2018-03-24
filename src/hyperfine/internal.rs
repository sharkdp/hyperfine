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
        OutputStyleOption::Basic => ProgressStyle::default_bar(),
        _ => ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template(" {spinner} {msg:<30} {wide_bar} ETA {eta_precise}"),
    };

    let progress_bar = match *option {
        OutputStyleOption::Basic => ProgressBar::hidden(),
        _ => ProgressBar::new(length),
    };
    progress_bar.set_style(progressbar_style.clone());
    progress_bar.enable_steady_tick(80);
    progress_bar.set_message(msg);

    progress_bar
}

/// A max function for f64's without NaNs
pub fn max(vals: &[f64]) -> f64 {
    *vals.iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap()
}

/// A min function for f64's without NaNs
pub fn min(vals: &[f64]) -> f64 {
    *vals.iter()
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

    println!("{}\n", "Summary".bold());
    println!("'{}' ran", fastest_item.command.cyan());
    longer_items.sort_by(|l, r| l.mean.partial_cmp(&r.mean).unwrap_or(Ordering::Equal));

    for item in longer_items {
        println!(
            "{} faster than '{}'",
            format!("{:8.2}x", item.mean / fastest_item.mean)
                .bold()
                .green(),
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
