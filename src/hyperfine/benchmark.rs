use std::io;
use std::process::{Command, Stdio};
use std::time::Instant;

use ansi_term::Colour::{Cyan, Green, Purple, White, Yellow};
use statistical::{mean, standard_deviation};

use hyperfine::internal::{get_progress_bar, max, min, HyperfineOptions, Second, Warnings,
                          MIN_EXECUTION_TIME};
use hyperfine::format::{format_duration, format_duration_unit, Unit};

/// Results from timing a single shell command
pub struct TimingResult {
    /// Execution time in seconds
    pub execution_time: Second,

    /// True if the command finished with exit code zero
    pub success: bool,
}

impl TimingResult {
    fn new(execution_time: Second, success: bool) -> TimingResult {
        TimingResult {
            execution_time,
            success,
        }
    }
}

/// Run the given shell command and measure the execution time
pub fn time_shell_command(
    shell_cmd: &str,
    ignore_failure: bool,
    shell_spawning_time: Option<Second>,
) -> io::Result<TimingResult> {
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

    let execution_time = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;

    // Correct for shell spawning time
    let execution_time_corrected = if let Some(spawning_time) = shell_spawning_time {
        if execution_time < spawning_time {
            0.0
        } else {
            execution_time - spawning_time
        }
    } else {
        execution_time
    };

    Ok(TimingResult::new(
        execution_time_corrected,
        status.success(),
    ))
}

/// Measure the average shell spawning time
pub fn mean_shell_spawning_time() -> io::Result<Second> {
    const COUNT: u64 = 200;
    let bar = get_progress_bar(COUNT, "Measuring shell spawning time");

    let mut times: Vec<Second> = vec![];
    for _ in 0..COUNT {
        // Just run the shell without any command
        let res = time_shell_command("", false, None);

        match res {
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Could not measure shell execution time. \
                     Make sure you can run 'sh -c \"\"'.",
                ))
            }
            Ok(r) => {
                times.push(r.execution_time);
            }
        }
        bar.inc(1);
    }
    bar.finish_and_clear();

    Ok(mean(&times))
}

/// Run the benchmark for a single shell command
pub fn run_benchmark(
    num: usize,
    cmd: &str,
    shell_spawning_time: Second,
    options: &HyperfineOptions,
) -> io::Result<()> {
    println!(
        "{}{}: {}",
        White.bold().paint("Benchmark #"),
        White.bold().paint((num + 1).to_string()),
        cmd
    );
    println!();

    let mut execution_times: Vec<Second> = vec![];
    let mut all_succeeded = true;

    // Warmup phase
    if options.warmup_count > 0 {
        let bar = get_progress_bar(options.warmup_count, "Performing warmup runs");

        for _ in 0..options.warmup_count {
            let _ = time_shell_command(cmd, options.ignore_failure, None)?;
            bar.inc(1);
        }
        bar.finish_and_clear();
    }

    // Set up progress bar (and spinner for initial measurement)
    let bar = get_progress_bar(options.min_runs, "Initial time measurement");

    // Run init / cleanup command
    let run_preparation_command = || {
        if let Some(ref preparation_command) = options.preparation_command {
            let _ = time_shell_command(preparation_command, options.ignore_failure, None);
        }
    };
    run_preparation_command();

    // Initial timing run
    let res = time_shell_command(cmd, options.ignore_failure, Some(shell_spawning_time))?;

    // Determine number of benchmark runs
    let runs_in_min_time =
        (options.min_time_sec / (res.execution_time + shell_spawning_time)) as u64;

    let count = if runs_in_min_time >= options.min_runs {
        runs_in_min_time
    } else {
        options.min_runs
    };

    let count_remaining = count - 1;

    // Save the first result
    execution_times.push(res.execution_time);
    all_succeeded = all_succeeded && res.success;

    // Re-configure the progress bar
    bar.set_length(count_remaining);

    // Gather statistics
    for _ in 0..count_remaining {
        run_preparation_command();

        let msg = {
            let mean = format_duration(mean(&execution_times), Unit::Auto);
            format!("Current estimate: {}", Green.paint(mean))
        };
        bar.set_message(&msg);

        let res = time_shell_command(cmd, options.ignore_failure, Some(shell_spawning_time))?;

        execution_times.push(res.execution_time);
        all_succeeded = all_succeeded && res.success;

        bar.inc(1);
    }
    bar.finish_and_clear();

    // Compute statistical quantities
    let t_mean = mean(&execution_times);
    let t_stddev = standard_deviation(&execution_times, Some(t_mean));
    let t_min = min(&execution_times);
    let t_max = max(&execution_times);

    // Formatting and console output
    let (mean_str, unit_mean) = format_duration_unit(t_mean, Unit::Auto);
    let stddev_str = format_duration(t_stddev, unit_mean);
    let min_str = format_duration(t_min, unit_mean);
    let max_str = format_duration(t_max, unit_mean);

    println!(
        "  Time ({} ± {}):     {} ± {}",
        Green.bold().paint("mean"),
        Green.paint("σ"),
        Green.bold().paint(mean_str),
        Green.paint(stddev_str)
    );
    println!(" ");

    println!(
        "  Range ({} … {}):   {} … {}",
        Cyan.paint("min"),
        Purple.paint("max"),
        Cyan.paint(min_str),
        Purple.paint(max_str)
    );

    // Warnings
    let mut warnings = vec![];

    // Check execution time
    if execution_times.iter().any(|&t| t < MIN_EXECUTION_TIME) {
        warnings.push(Warnings::FastExecutionTime);
    }

    // Check programm exit codes
    if !all_succeeded {
        warnings.push(Warnings::NonZeroExitCode);
    }

    if warnings.len() > 0 {
        eprintln!(" ");
        for warning in &warnings {
            eprintln!("  {}: {}", Yellow.paint("Warning"), warning);
        }
    }

    println!(" ");

    Ok(())
}
