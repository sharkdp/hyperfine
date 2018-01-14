extern crate ansi_term;
#[macro_use]
extern crate clap;
extern crate indicatif;

use std::cmp;
use std::error::Error;
use std::io;
use std::process::{Command, Stdio};
use std::time::Instant;
use std::fmt;

use indicatif::{ProgressBar, ProgressStyle};
use ansi_term::Colour::{Cyan, Green, Red, White, Yellow};
use clap::{App, AppSettings, Arg};

/// Type alias for unit of time
type Second = f64;

/// Threshold for warning about fast execution time
const MIN_EXECUTION_TIME: Second = 5e-3;

/// Print error message to stderr and terminate
pub fn error(message: &str) -> ! {
    eprintln!("{}: {}", Red.paint("Error"), message);
    std::process::exit(1);
}

struct CmdResult {
    /// Execution time in seconds
    execution_time_sec: Second,

    /// True if the command finished with exit code zero
    success: bool,
}

impl CmdResult {
    fn new(execution_time_sec: Second, success: bool) -> CmdResult {
        CmdResult {
            execution_time_sec,
            success,
        }
    }
}

/// Run the given shell command and measure the execution time
fn time_shell_command(shell_cmd: &str, ignore_failure: bool) -> io::Result<CmdResult> {
    let start = Instant::now();

    let status = Command::new("sh")
        .arg("-c")
        .arg(shell_cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !ignore_failure && !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Command terminated with non-zero exit code. \
             Use the '-i'/'--ignore-failure' option if you want to ignore this.",
        ));
    }

    let duration = start.elapsed();

    let execution_time_sec = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;

    Ok(CmdResult::new(execution_time_sec, status.success()))
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

/// Possible benchmark warnings
enum Warnings {
    FastExecutionTime,
    NonZeroExitCode,
}

impl fmt::Display for Warnings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Warnings::FastExecutionTime => write!(
                f,
                "Command took less than {:.0} ms to complete. Results might be inaccurate.",
                MIN_EXECUTION_TIME * 1e3
            ),
            &Warnings::NonZeroExitCode => write!(f, "Ignoring non-zero exit code."),
        }
    }
}

/// Calculate statistical average
fn mean<I>(values: I) -> f64
where
    I: IntoIterator<Item = f64>,
{
    let mut sum: f64 = 0.0;
    let mut len: u64 = 0;
    for v in values {
        sum += v;
        len += 1;
    }
    sum / (len as f64)
}

/// Run the benchmark for a single shell command
fn run_benchmark(
    num: usize,
    cmd: &str,
    shell_spawning_time: Second,
    options: &HyperfineOptions,
) -> io::Result<()> {
    println!(
        "{}{}: {}",
        White.bold().paint("Benchmark #"),
        White.bold().paint((num + 1).to_string()),
        Cyan.paint(cmd)
    );
    println!();

    let mut results = vec![];

    // Warmup phase
    if options.warmup_count > 0 {
        let bar = get_progress_bar(options.warmup_count, "Performing warmup runs");

        for _ in 0..options.warmup_count {
            let _ = time_shell_command(cmd, options.ignore_failure)?;
            bar.inc(1);
        }
        bar.finish_and_clear();
    }

    // Set up progress bar (and spinner for initial measurement)
    let bar = get_progress_bar(options.min_runs, "Initial time measurement");

    // Initial timing run
    let res = time_shell_command(cmd, options.ignore_failure)?;

    // Determine number of benchmark runs
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
    for _ in 0..count {
        let res = time_shell_command(cmd, options.ignore_failure)?;
        results.push(res);
        bar.inc(1);
    }
    bar.finish_and_clear();

    // Compute statistical quantities
    let get_execution_time = |r: &CmdResult| -> Second {
        if r.execution_time_sec < shell_spawning_time {
            0.0
        } else {
            r.execution_time_sec - shell_spawning_time
        }
    };

    // Iterator over (corrected) execution times
    let execution_times = results.iter().map(&get_execution_time);

    let t_mean = mean(execution_times.clone());
    let t2_mean = mean(execution_times.clone().map(|t| t.powi(2)));

    let stddev = (t2_mean - t_mean.powi(2)).sqrt();

    // Formatting and console output
    let time_fmt = format!("{:.3} s ± {:.3} s", t_mean, stddev);

    println!("  Time: {}", Green.paint(time_fmt));

    // Warnings
    let mut warnings = vec![];

    // Check execution time
    if execution_times.clone().any(|t| t < MIN_EXECUTION_TIME) {
        warnings.push(Warnings::FastExecutionTime);
    }

    // Check programm exit codes
    if results.iter().any(|r| !r.success) {
        warnings.push(Warnings::NonZeroExitCode);
    }

    if warnings.len() > 0 {
        eprintln!();
        for warning in &warnings {
            eprintln!("  {}: {}", Yellow.paint("Warning"), warning);
        }
    }

    println!();

    Ok(())
}

// Measure the average shell spawning time
fn mean_shell_spawning_time() -> io::Result<Second> {
    const COUNT: u64 = 200;
    let bar = get_progress_bar(COUNT, "Measuring shell spawning time");

    let mut times: Vec<Second> = vec![];
    for _ in 0..COUNT {
        let res = time_shell_command("", false);

        match res {
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Could not measure shell execution time. \
                     Make sure you can run 'sh -c \"\"'.",
                ))
            }
            Ok(r) => {
                times.push(r.execution_time_sec);
            }
        }
        bar.inc(1);
    }
    bar.finish_and_clear();

    let mean: f64 = mean(times.iter().cloned()); // TODO: get rid of .cloned()
    Ok(mean)
}

/// Runs the benchmark for the given commands
fn run(commands: &Vec<&str>, options: &HyperfineOptions) -> io::Result<()> {
    let shell_spawning_time = mean_shell_spawning_time()?;

    // Run the benchmarks
    for (num, cmd) in commands.iter().enumerate() {
        run_benchmark(num, &cmd, shell_spawning_time, &options)?;
    }

    Ok(())
}

pub struct HyperfineOptions {
    /// Number of warmup runs
    pub warmup_count: u64,

    /// Minimum number of benchmark runs
    pub min_runs: u64,

    /// Minimum benchmarking time
    pub min_time_sec: Second,

    /// Whether or not to ignore non-zero exit codes
    pub ignore_failure: bool,
}

impl Default for HyperfineOptions {
    fn default() -> HyperfineOptions {
        HyperfineOptions {
            warmup_count: 0,
            min_runs: 10,
            min_time_sec: 3.0,
            ignore_failure: false,
        }
    }
}

fn main() {
    let matches = App::new("hyperfine")
        .version(crate_version!())
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .about("A command-line benchmarking tool")
        .arg(
            Arg::with_name("command")
                .help("Command to benchmark")
                .required(true)
                .multiple(true)
                .empty_values(false),
        )
        .arg(
            Arg::with_name("warmup")
                .long("warmup")
                .short("w")
                .takes_value(true)
                .value_name("NUM")
                .help("Perform NUM warmup runs before the actual benchmark"),
        )
        .arg(
            Arg::with_name("min-runs")
                .long("min-runs")
                .short("m")
                .takes_value(true)
                .value_name("NUM")
                .help("Perform at least NUM runs for each command"),
        )
        .arg(
            Arg::with_name("ignore-failure")
                .long("ignore-failure")
                .short("i")
                .help("Ignore non-zero exit codes"),
        )
        .get_matches();

    let str_to_u64 = |n| u64::from_str_radix(n, 10).ok();

    // Process command line options
    let mut options = HyperfineOptions::default();

    options.warmup_count = matches
        .value_of("warmup")
        .and_then(&str_to_u64)
        .unwrap_or(0);

    if let Some(min_runs) = matches.value_of("min-runs").and_then(&str_to_u64) {
        options.min_runs = cmp::max(1, min_runs);
    }

    options.ignore_failure = matches.is_present("ignore-failure");

    let commands = matches.values_of("command").unwrap().collect();
    let res = run(&commands, &options);

    match res {
        Err(e) => error(e.description()),
        Ok(_) => {}
    }
}
