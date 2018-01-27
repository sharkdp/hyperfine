use std::io;
use std::process::{Command, Stdio};
use std::time::Instant;

use colored::*;
use statistical::{mean, standard_deviation};

use hyperfine::internal::{get_progress_bar, max, min, CmdFailureAction, HyperfineOptions,
                          OutputStyleOption, Second, MIN_EXECUTION_TIME};
use hyperfine::warnings::Warnings;
use hyperfine::format::{format_duration, format_duration_unit};
use hyperfine::cputime::{cpu_time_interval, get_cpu_times};
use hyperfine::outlier_detection::{modified_zscores, OUTLIER_THRESHOLD};

/// Results from timing a single shell command
#[derive(Debug, Copy, Clone)]
pub struct TimingResult {
    /// Wall clock time
    pub time_real: Second,

    /// Time spent in user mode
    pub time_user: Second,

    /// Time spent in kernel mode
    pub time_system: Second,
}

/// Correct for shell spawning time
fn subtract_shell_spawning_time(time: Second, shell_spawning_time: Second) -> Second {
    if time < shell_spawning_time {
        0.0
    } else {
        time - shell_spawning_time
    }
}

/// Run the given shell command and measure the execution time
pub fn time_shell_command(
    shell_cmd: &str,
    failure_action: CmdFailureAction,
    shell_spawning_time: Option<TimingResult>,
) -> io::Result<(TimingResult, bool)> {
    let start = Instant::now();
    let start_cpu = get_cpu_times();

    let status = Command::new("sh")
        .arg("-c")
        .arg(shell_cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    let end_cpu = get_cpu_times();
    let duration = start.elapsed();

    if failure_action == CmdFailureAction::RaiseError && !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Command terminated with non-zero exit code. \
             Use the '-i'/'--ignore-failure' option if you want to ignore this.",
        ));
    }

    // Real time
    let mut time_real = duration.as_secs() as f64 + (duration.subsec_nanos() as f64) * 1e-9;

    // User and system time
    let cpu_interval = cpu_time_interval(&start_cpu, &end_cpu);
    let mut time_user = cpu_interval.user;
    let mut time_system = cpu_interval.system;

    // Correct for shell spawning time
    if let Some(spawning_time) = shell_spawning_time {
        time_real = subtract_shell_spawning_time(time_real, spawning_time.time_real);
        time_user = subtract_shell_spawning_time(cpu_interval.user, spawning_time.time_user);
        time_system = subtract_shell_spawning_time(cpu_interval.system, spawning_time.time_system);
    }

    Ok((
        TimingResult {
            time_real,
            time_user,
            time_system,
        },
        status.success(),
    ))
}

/// Measure the average shell spawning time
pub fn mean_shell_spawning_time(style: &OutputStyleOption) -> io::Result<TimingResult> {
    const COUNT: u64 = 200;
    let progress_bar = get_progress_bar(COUNT, "Measuring shell spawning time", style);

    let mut times_real: Vec<Second> = vec![];
    let mut times_user: Vec<Second> = vec![];
    let mut times_system: Vec<Second> = vec![];

    for _ in 0..COUNT {
        // Just run the shell without any command
        let res = time_shell_command("", CmdFailureAction::RaiseError, None);

        match res {
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Could not measure shell execution time. \
                     Make sure you can run 'sh -c \"\"'.",
                ))
            }
            Ok((r, _)) => {
                times_real.push(r.time_real);
                times_user.push(r.time_user);
                times_system.push(r.time_system);
            }
        }
        progress_bar.inc(1);
    }
    progress_bar.finish_and_clear();

    Ok(TimingResult {
        time_real: mean(&times_real),
        time_user: mean(&times_user),
        time_system: mean(&times_system),
    })
}

/// Run the command specified by `--prepare`.
fn run_preparation_command(command: &Option<String>) -> io::Result<()> {
    if let &Some(ref preparation_command) = command {
        let res = time_shell_command(preparation_command, CmdFailureAction::RaiseError, None);
        if let Err(_) = res {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "The preparation command terminated with a non-zero exit code. \
                 Append ' || true' to the command if you are sure that this can be ignored.",
            ));
        }
    }
    Ok(())
}

/// Run the benchmark for a single shell command
pub fn run_benchmark(
    num: usize,
    cmd: &str,
    shell_spawning_time: TimingResult,
    options: &HyperfineOptions,
) -> io::Result<()> {
    let (benchark_text, current_num) = match options.output_style {
        OutputStyleOption::Basic => ("Benchmark #".white(), (num + 1).to_string().white()),
        OutputStyleOption::Full => ("Benchmark #".bold(), (num + 1).to_string().bold()),
    };

    println!("{}{}: {}", benchark_text, current_num, cmd);
    println!();

    let mut times_real: Vec<Second> = vec![];
    let mut times_user: Vec<Second> = vec![];
    let mut times_system: Vec<Second> = vec![];
    let mut all_succeeded = true;

    // Warmup phase
    if options.warmup_count > 0 {
        let progress_bar = get_progress_bar(
            options.warmup_count,
            "Performing warmup runs",
            &options.output_style,
        );

        for _ in 0..options.warmup_count {
            let _ = time_shell_command(cmd, options.failure_action, None)?;
            progress_bar.inc(1);
        }
        progress_bar.finish_and_clear();
    }

    // Set up progress bar (and spinner for initial measurement)
    let progress_bar = get_progress_bar(
        options.min_runs,
        "Initial time measurement",
        &options.output_style,
    );

    // Run init / cleanup command
    run_preparation_command(&options.preparation_command)?;

    // Initial timing run
    let (res, success) =
        time_shell_command(cmd, options.failure_action, Some(shell_spawning_time))?;

    // Determine number of benchmark runs
    let runs_in_min_time =
        (options.min_time_sec / (res.time_real + shell_spawning_time.time_real)) as u64;

    let count = if runs_in_min_time >= options.min_runs {
        runs_in_min_time
    } else {
        options.min_runs
    };

    let count_remaining = count - 1;

    // Save the first result
    times_real.push(res.time_real);
    times_user.push(res.time_user);
    times_system.push(res.time_system);

    all_succeeded = all_succeeded && success;

    // Re-configure the progress bar
    progress_bar.set_length(count_remaining);

    // Gather statistics
    for _ in 0..count_remaining {
        run_preparation_command(&options.preparation_command)?;

        let msg = {
            let mean = format_duration(mean(&times_real), None);
            format!("Current estimate: {}", mean.to_string().green())
        };
        progress_bar.set_message(&msg);

        let (res, success) =
            time_shell_command(cmd, options.failure_action, Some(shell_spawning_time))?;

        times_real.push(res.time_real);
        times_user.push(res.time_user);
        times_system.push(res.time_system);

        all_succeeded = all_succeeded && success;

        progress_bar.inc(1);
    }
    progress_bar.finish_and_clear();

    // Compute statistical quantities
    let t_mean = mean(&times_real);
    let t_stddev = standard_deviation(&times_real, Some(t_mean));
    let t_min = min(&times_real);
    let t_max = max(&times_real);

    let user_mean = mean(&times_user);
    let system_mean = mean(&times_system);

    // Formatting and console output
    let (mean_str, unit_mean) = format_duration_unit(t_mean, None);
    let stddev_str = format_duration(t_stddev, Some(unit_mean));
    let min_str = format_duration(t_min, Some(unit_mean));
    let max_str = format_duration(t_max, Some(unit_mean));

    let (user_str, user_unit) = format_duration_unit(user_mean, None);
    let system_str = format_duration(system_mean, Some(user_unit));

    let (mean_start, delta, mean_val, stddev_val, user_val, sys_val) = match options.output_style {
        OutputStyleOption::Basic => (
            "mean".white(),
            "σ".white(),
            mean_str.white(),
            stddev_str.white(),
            user_str.white(),
            system_str.white(),
        ),
        OutputStyleOption::Full => (
            "mean".green().bold(),
            "σ".green(),
            mean_str.green().bold(),
            stddev_str.green(),
            user_str.blue(),
            system_str.blue(),
        ),
    };

    println!(
        "  Time ({} ± {}):     {:>8} ± {:>8}    [User: {}, System: {}]",
        mean_start, delta, mean_val, stddev_val, user_val, sys_val
    );
    println!(" ");

    let (min_start, max_start, min_val, max_val) = match options.output_style {
        OutputStyleOption::Basic => (
            "min".white(),
            "max".white(),
            min_str.white(),
            max_str.white(),
        ),
        OutputStyleOption::Full => (
            "min".cyan(),
            "max".purple(),
            min_str.cyan(),
            max_str.purple(),
        ),
    };

    println!(
        "  Range ({} … {}):   {:>8} … {:>8}",
        min_start, max_start, min_val, max_val
    );

    // Warnings
    let mut warnings = vec![];

    // Check execution time
    if times_real.iter().any(|&t| t < MIN_EXECUTION_TIME) {
        warnings.push(Warnings::FastExecutionTime);
    }

    // Check programm exit codes
    if !all_succeeded {
        warnings.push(Warnings::NonZeroExitCode);
    }

    // Run outlier detection
    let scores = modified_zscores(&times_real);
    if scores[0] > OUTLIER_THRESHOLD {
        warnings.push(Warnings::SlowInitialRun(times_real[0]));
    } else if scores.iter().any(|&s| s > OUTLIER_THRESHOLD) {
        warnings.push(Warnings::OutliersDetected);
    }

    if !warnings.is_empty() {
        eprintln!(" ");
        let warning_text = match options.output_style {
            OutputStyleOption::Basic => "Warning".white(),
            OutputStyleOption::Full => "Warning".yellow(),
        };

        for warning in &warnings {
            eprintln!("  {}: {}", warning_text, warning);
        }
    }

    println!(" ");

    Ok(())
}
