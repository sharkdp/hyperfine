use std::io;
use std::process::{Command, Stdio};
use std::time::Instant;

use ansi_term::Colour::{Blue, Cyan, Green, Purple, White, Yellow};
use statistical::{mean, standard_deviation};

use hyperfine::internal::{get_progress_bar, max, min, CmdFailureAction, HyperfineOptions, Second,
                          Warnings, MIN_EXECUTION_TIME};
use hyperfine::format::{format_duration, format_duration_unit};
use hyperfine::cputime::{cpu_time_interval, get_cpu_times};

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
pub fn mean_shell_spawning_time() -> io::Result<TimingResult> {
    const COUNT: u64 = 200;
    let progress_bar = get_progress_bar(COUNT, "Measuring shell spawning time");

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
    println!(
        "{}{}: {}",
        White.bold().paint("Benchmark #"),
        White.bold().paint((num + 1).to_string()),
        cmd
    );
    println!();

    let mut times_real: Vec<Second> = vec![];
    let mut times_user: Vec<Second> = vec![];
    let mut times_system: Vec<Second> = vec![];
    let mut all_succeeded = true;

    // Warmup phase
    if options.warmup_count > 0 {
        let progress_bar = get_progress_bar(options.warmup_count, "Performing warmup runs");

        for _ in 0..options.warmup_count {
            let _ = time_shell_command(cmd, options.failure_action, None)?;
            progress_bar.inc(1);
        }
        progress_bar.finish_and_clear();
    }

    // Set up progress bar (and spinner for initial measurement)
    let progress_bar = get_progress_bar(options.min_runs, "Initial time measurement");

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
            format!("Current estimate: {}", Green.paint(mean))
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

    println!(
        "  Time ({} ± {}):     {} ± {}    [User: {}, System: {}]",
        Green.bold().paint("mean"),
        Green.paint("σ"),
        Green.bold().paint(mean_str),
        Green.paint(stddev_str),
        Blue.paint(user_str),
        Blue.paint(system_str)
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
    if times_real.iter().any(|&t| t < MIN_EXECUTION_TIME) {
        warnings.push(Warnings::FastExecutionTime);
    }

    // Check programm exit codes
    if !all_succeeded {
        warnings.push(Warnings::NonZeroExitCode);
    }

    if !warnings.is_empty() {
        eprintln!(" ");
        for warning in &warnings {
            eprintln!("  {}: {}", Yellow.paint("Warning"), warning);
        }
    }

    println!(" ");

    Ok(())
}
