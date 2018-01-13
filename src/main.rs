extern crate ansi_term;
extern crate clap;
extern crate indicatif;

use std::process::{Command, Stdio};
use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};
use ansi_term::Colour::{Cyan, Green, Red};
use clap::{App, AppSettings, Arg};

struct CmdResult {
    /// Execution time in seconds
    execution_time_sec: f64,

    /// True if the command finished with exit code zero
    success: bool,
}

impl CmdResult {
    fn new(execution_time_sec: f64, success: bool) -> CmdResult {
        CmdResult {
            execution_time_sec,
            success,
        }
    }
}

/// Run the given shell command and measure the execution time
fn time_shell_command(shell_cmd: &str) -> CmdResult {
    let start = Instant::now();

    let status = Command::new("sh")
        .arg("-c")
        .arg(shell_cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("failed to execute process");

    let duration = start.elapsed();

    let execution_time_sec = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;

    CmdResult::new(execution_time_sec, status.success())
}

/// Return a pre-configured progress bar
fn get_progress_bar(length: u64, msg: &str) -> ProgressBar {
    let progressbar_style = ProgressStyle::default_spinner()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        .template(" {spinner} {msg:<28} {wide_bar} ETA {eta_precise}");

    let bar = ProgressBar::new(length);
    bar.set_style(progressbar_style.clone());
    bar.enable_steady_tick(80);
    bar.set_message(msg);

    bar
}

/// Run the benchmark for a single shell command
fn run_benchmark(cmd: &str, options: &HyperfineOptions) {
    println!("Command: {}", Cyan.paint(cmd));
    println!();

    let mut results = vec![];

    // Warmup phase
    if let Some(warmup_count) = options.warmup_count
    {
        let bar = get_progress_bar(warmup_count, "Performing warmup runs");

        for _ in 1..warmup_count {
            bar.inc(1);
            let _ = time_shell_command(cmd);
        }
        bar.finish_and_clear();
    }

    // Set up progress bar (and spinner for initial measurement)
    let bar = get_progress_bar(options.min_runs, "Initial time measurement");

    // Initial timing run
    let res = time_shell_command(cmd);

    let runs_in_min_time = (options.min_time_sec / res.execution_time_sec) as u64;

    let count = if runs_in_min_time >= options.min_runs {
        runs_in_min_time
    } else {
        options.min_runs
    };

    // Save the first result
    results.push(res);

    // Re-configure the progress bar
    bar.set_length(count);
    bar.set_message("Collecting statistics");

    // Gather statistics
    for _ in 1..count {
        bar.inc(1);
        let res = time_shell_command(cmd);
        results.push(res);
    }
    bar.finish_and_clear();

    // Compute statistical quantities
    let t_sum: f64 = results.iter().map(|r| r.execution_time_sec).sum();
    let t_mean = t_sum / (results.len() as f64);

    let t2_sum: f64 = results.iter().map(|r| r.execution_time_sec.powi(2)).sum();
    let t2_mean = t2_sum / (results.len() as f64);

    let stddev = (t2_mean - t_mean.powi(2)).sqrt();

    // Formatting and console output
    let time_fmt = format!("{:.3} s ± {:.3} s", t_mean, stddev);

    println!("  Time: {}", Green.paint(time_fmt));

    if !results.iter().all(|r| r.success) {
        println!(
            "  {}: Program returned non-zero exit status",
            Red.paint("Warning")
        );
    };

    println!();
}

pub struct HyperfineOptions {
    pub warmup_count: Option<u64>,
    pub min_runs: u64,
    pub min_time_sec: f64,
}

impl Default for HyperfineOptions {
    fn default() -> HyperfineOptions {
        HyperfineOptions {
            warmup_count: None,
            min_runs: 10,
            min_time_sec: 5.0
        }
    }
}

fn main() {
    let matches = App::new("hyperfine")
        .global_settings(&[AppSettings::ColoredHelp])
        .version("0.1")
        .about("A command-line benchmarking tool")
        .author("David Peter <mail@david-peter.de>")
        .arg(
            Arg::with_name("command")
                .help("Command to benchmark")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("warmup")
                .long("warmup")
                .short("w")
                .takes_value(true)
                .value_name("NUM")
                .help("Perform NUM warmup runs before the actual benchmark"),
        )
        .get_matches();


    let mut options = HyperfineOptions::default();
    options.warmup_count =
        matches.value_of("warmup")
        .and_then(|n| u64::from_str_radix(n, 10).ok());

    let commands = matches.values_of("command").unwrap();
    for cmd in commands {
        run_benchmark(&cmd, &options);
    }
}
