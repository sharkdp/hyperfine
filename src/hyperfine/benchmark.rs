use std::io;
use std::process::{Command, Stdio};
use std::time::Instant;

use ansi_term::Colour::{Cyan, Green, White, Yellow};

use hyperfine::internal::{get_progress_bar, HyperfineOptions, Second, Warnings, MIN_EXECUTION_TIME};
use hyperfine::statistics::mean;

/// Results from timing a single shell command
pub struct TimingResult {
    /// Execution time in seconds
    pub execution_time_sec: Second,

    /// True if the command finished with exit code zero
    pub success: bool,
}

impl TimingResult {
    fn new(execution_time_sec: Second, success: bool) -> TimingResult {
        TimingResult {
            execution_time_sec,
            success,
        }
    }
}

/// Run the given shell command and measure the execution time
pub fn time_shell_command(shell_cmd: &str, ignore_failure: bool) -> io::Result<TimingResult> {
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

    Ok(TimingResult::new(execution_time_sec, status.success()))
}

/// Measure the average shell spawning time
pub fn mean_shell_spawning_time() -> io::Result<Second> {
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

/// Run the benchmark for a single shell command
pub fn run_benchmark(
    num: usize,
    cmd: &str,
    shell_spawning_time: Second,
    options: &HyperfineOptions,
) -> io::Result<()> {
    // Helper function to compute corrected execution times
    let get_execution_time = |r: &TimingResult| -> Second {
        if r.execution_time_sec < shell_spawning_time {
            0.0
        } else {
            r.execution_time_sec - shell_spawning_time
        }
    };

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

    let count_remaining = count - 1;

    // Save the first result
    results.push(res);

    // Re-configure the progress bar
    bar.set_length(count_remaining);

    // Gather statistics
    for _ in 0..count_remaining {
        let msg = {
            let execution_times = results.iter().map(&get_execution_time);
            let mean = format!("{:.3} s", mean(execution_times));
            format!("Current estimate: {:.3}", Green.paint(mean))
        };
        bar.set_message(&msg);

        let res = time_shell_command(cmd, options.ignore_failure)?;
        results.push(res);
        bar.inc(1);
    }
    bar.finish_and_clear();

    // Compute statistical quantities

    // Corrected execution times
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
