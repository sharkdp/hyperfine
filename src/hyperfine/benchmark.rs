use std::cmp;
use std::io;
use std::process::Stdio;

use colored::*;
use statistical::{mean, standard_deviation};

use hyperfine::format::{format_duration, format_duration_unit};
use hyperfine::internal::{get_progress_bar, max, min, MIN_EXECUTION_TIME};
use hyperfine::outlier_detection::{modified_zscores, OUTLIER_THRESHOLD};
use hyperfine::shell::execute_and_time;
use hyperfine::timer::wallclocktimer::WallClockTimer;
use hyperfine::timer::{TimerStart, TimerStop};
use hyperfine::types::{BenchmarkResult, Command};
use hyperfine::types::{CmdFailureAction, HyperfineOptions, OutputStyleOption, Second};
use hyperfine::warnings::Warnings;

/// Results from timing a single shell command
#[derive(Debug, Default, Copy, Clone)]
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
    shell: &str,
    command: &Command,
    show_output: bool,
    failure_action: CmdFailureAction,
    shell_spawning_time: Option<TimingResult>,
) -> io::Result<(TimingResult, bool)> {
    let wallclock_timer = WallClockTimer::start();

    let (stdout, stderr) = if show_output {
        (Stdio::inherit(), Stdio::inherit())
    } else {
        (Stdio::null(), Stdio::null())
    };
    let result = execute_and_time(stdout, stderr, &command.get_shell_command(), shell)?;

    let mut time_user = result.user_time;
    let mut time_system = result.system_time;

    let mut time_real = wallclock_timer.stop();

    if failure_action == CmdFailureAction::RaiseError && !result.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Command terminated with non-zero exit code. \
             Use the '-i'/'--ignore-failure' option if you want to ignore this. \
             Alternatively, use the '--show-output' option to debug what went wrong.",
        ));
    }

    // Correct for shell spawning time
    if let Some(spawning_time) = shell_spawning_time {
        time_real = subtract_shell_spawning_time(time_real, spawning_time.time_real);
        time_user = subtract_shell_spawning_time(time_user, spawning_time.time_user);
        time_system = subtract_shell_spawning_time(time_system, spawning_time.time_system);
    }

    Ok((
        TimingResult {
            time_real,
            time_user,
            time_system,
        },
        result.status.success(),
    ))
}

/// Measure the average shell spawning time
pub fn mean_shell_spawning_time(
    shell: &str,
    style: &OutputStyleOption,
    show_output: bool,
) -> io::Result<TimingResult> {
    const COUNT: u64 = 200;
    let progress_bar = get_progress_bar(COUNT, "Measuring shell spawning time", style);

    let mut times_real: Vec<Second> = vec![];
    let mut times_user: Vec<Second> = vec![];
    let mut times_system: Vec<Second> = vec![];

    for _ in 0..COUNT {
        // Just run the shell without any command
        let res = time_shell_command(
            shell,
            &Command::new(""),
            show_output,
            CmdFailureAction::RaiseError,
            None,
        );

        match res {
            Err(_) => {
                let shell_cmd = if cfg!(windows) {
                    format!("{} /C \"\"", shell)
                } else {
                    format!("{} -c \"\"", shell)
                };

                return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Could not measure shell execution time. \
                        Make sure you can run '{}'.", shell_cmd),
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
fn run_preparation_command(
    shell: &str,
    command: &Option<Command>,
    show_output: bool,
) -> io::Result<TimingResult> {
    if let &Some(ref cmd) = command {
        let res = time_shell_command(shell, cmd, show_output, CmdFailureAction::RaiseError, None);
        if res.is_err() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "The preparation command terminated with a non-zero exit code. \
                 Append ' || true' to the command if you are sure that this can be ignored.",
            ));
        }
        return res.map(|r| r.0);
    }
    Ok(TimingResult {
        ..Default::default()
    })
}

/// Run the benchmark for a single shell command
pub fn run_benchmark(
    num: usize,
    cmd: &Command,
    shell_spawning_time: TimingResult,
    options: &HyperfineOptions,
) -> io::Result<BenchmarkResult> {
    println!(
        "{}{}: {}",
        "Benchmark #".bold(),
        (num + 1).to_string().bold(),
        cmd
    );

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
            let _ = time_shell_command(&options.shell, cmd, options.show_output, options.failure_action, None)?;
            progress_bar.inc(1);
        }
        progress_bar.finish_and_clear();
    }

    // Set up progress bar (and spinner for initial measurement)
    let progress_bar = get_progress_bar(
        options.runs.min,
        "Initial time measurement",
        &options.output_style,
    );

    // Run init / cleanup command
    let prepare_cmd = options
        .preparation_command
        .as_ref()
        .map(|preparation_command| match cmd.get_parameter() {
            Some((param, value)) => Command::new_parametrized(preparation_command, param, value),
            None => Command::new(preparation_command),
        });
    let prepare_res = run_preparation_command(&options.shell, &prepare_cmd, options.show_output)?;

    // Initial timing run
    let (res, success) = time_shell_command(
        &options.shell,
        cmd,
        options.show_output,
        options.failure_action,
        Some(shell_spawning_time),
    )?;

    // Determine number of benchmark runs
    let runs_in_min_time = (options.min_time_sec
        / (res.time_real + prepare_res.time_real + shell_spawning_time.time_real))
        as u64;

    let count = {
        let min = cmp::max(runs_in_min_time, options.runs.min);

        options
            .runs
            .max
            .as_ref()
            .map(|max| cmp::min(min, *max))
            .unwrap_or(min)
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
        run_preparation_command(&options.shell, &prepare_cmd, options.show_output)?;

        let msg = {
            let mean = format_duration(mean(&times_real), options.time_unit);
            format!("Current estimate: {}", mean.to_string().green())
        };
        progress_bar.set_message(&msg);

        let (res, success) = time_shell_command(
            &options.shell,
            cmd,
            options.show_output,
            options.failure_action,
            Some(shell_spawning_time),
        )?;

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
    let (mean_str, unit_mean) = format_duration_unit(t_mean, options.time_unit);
    let stddev_str = format_duration(t_stddev, Some(unit_mean));
    let min_str = format_duration(t_min, Some(unit_mean));
    let max_str = format_duration(t_max, Some(unit_mean));

    let (user_str, user_unit) = format_duration_unit(user_mean, None);
    let system_str = format_duration(system_mean, Some(user_unit));

    println!(
        "  Time ({} ± {}):     {:>8} ± {:>8}    [User: {}, System: {}]",
        "mean".green().bold(),
        "σ".green(),
        mean_str.green().bold(),
        stddev_str.green(),
        user_str.blue(),
        system_str.blue()
    );

    println!(
        "  Range ({} … {}):   {:>8} … {:>8}",
        "min".cyan(),
        "max".purple(),
        min_str.cyan(),
        max_str.purple()
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

        for warning in &warnings {
            eprintln!("  {}: {}", "Warning".yellow(), warning);
        }
    }

    println!(" ");

    Ok(BenchmarkResult::new(
        cmd.get_shell_command(),
        t_mean,
        t_stddev,
        user_mean,
        system_mean,
        t_min,
        t_max,
        times_real,
    ))
}
