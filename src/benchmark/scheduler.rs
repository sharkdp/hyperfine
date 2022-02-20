use std::cmp;
use std::process::{ExitStatus, Stdio};

use colored::*;
use statistical::{mean, median, standard_deviation};

use super::benchmark_result::BenchmarkResult;
use super::timing_result::TimingResult;
use super::{relative_speed, MIN_EXECUTION_TIME};

use crate::command::Command;
use crate::command::Commands;
use crate::export::ExportManager;
use crate::options::{CmdFailureAction, CommandOutputPolicy, Options, OutputStyleOption, Shell};
use crate::outlier_detection::{modified_zscores, OUTLIER_THRESHOLD};
use crate::output::format::{format_duration, format_duration_unit};
use crate::output::progress_bar::get_progress_bar;
use crate::output::warnings::Warnings;
use crate::parameter::ParameterNameAndValue;
use crate::shell::execute_and_time;
use crate::timer::wallclocktimer::WallClockTimer;
use crate::timer::{TimerStart, TimerStop};
use crate::util::exit_code::extract_exit_code;
use crate::util::min_max::{max, min};
use crate::util::units::Second;

use anyhow::{bail, Result};

///////////////

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
    shell: &Shell,
    command: &Command<'_>,
    command_output_policy: CommandOutputPolicy,
    failure_action: CmdFailureAction,
    shell_spawning_time: Option<TimingResult>,
) -> Result<(TimingResult, ExitStatus)> {
    let (stdout, stderr) = match command_output_policy {
        CommandOutputPolicy::Discard => (Stdio::null(), Stdio::null()),
        CommandOutputPolicy::Forward => (Stdio::inherit(), Stdio::inherit()),
    };

    let wallclock_timer = WallClockTimer::start();
    let result = execute_and_time(stdout, stderr, &command.get_shell_command(), shell)?;
    let mut time_real = wallclock_timer.stop();

    let mut time_user = result.user_time;
    let mut time_system = result.system_time;

    if failure_action == CmdFailureAction::RaiseError && !result.status.success() {
        bail!(
            "{}. Use the '-i'/'--ignore-failure' option if you want to ignore this. \
                Alternatively, use the '--show-output' option to debug what went wrong.",
            result.status.code().map_or(
                "The process has been terminated by a signal".into(),
                |c| format!("Command terminated with non-zero exit code: {}", c)
            )
        );
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
        result.status,
    ))
}

/// Measure the average shell spawning time
pub fn mean_shell_spawning_time(
    shell: &Shell,
    style: OutputStyleOption,
    command_output_policy: CommandOutputPolicy,
) -> Result<TimingResult> {
    const COUNT: u64 = 50;
    let progress_bar = if style != OutputStyleOption::Disabled {
        Some(get_progress_bar(
            COUNT,
            "Measuring shell spawning time",
            style,
        ))
    } else {
        None
    };

    let mut times_real: Vec<Second> = vec![];
    let mut times_user: Vec<Second> = vec![];
    let mut times_system: Vec<Second> = vec![];

    for _ in 0..COUNT {
        // Just run the shell without any command
        let res = time_shell_command(
            shell,
            &Command::new(None, ""),
            command_output_policy,
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

                bail!(
                    "Could not measure shell execution time. Make sure you can run '{}'.",
                    shell_cmd
                );
            }
            Ok((r, _)) => {
                times_real.push(r.time_real);
                times_user.push(r.time_user);
                times_system.push(r.time_system);
            }
        }

        if let Some(bar) = progress_bar.as_ref() {
            bar.inc(1)
        }
    }

    if let Some(bar) = progress_bar.as_ref() {
        bar.finish_and_clear()
    }

    Ok(TimingResult {
        time_real: mean(&times_real),
        time_user: mean(&times_user),
        time_system: mean(&times_system),
    })
}
///////////////

pub struct Scheduler<'a> {
    commands: &'a Commands<'a>,
    options: &'a Options,
    export_manager: &'a ExportManager,
    results: Vec<BenchmarkResult>,
}

impl<'a> Scheduler<'a> {
    pub fn new(
        commands: &'a Commands,
        options: &'a Options,
        export_manager: &'a ExportManager,
    ) -> Self {
        Self {
            commands,
            options,
            export_manager,
            results: vec![],
        }
    }

    pub fn run_benchmarks(&mut self) -> Result<()> {
        let shell_spawning_time = mean_shell_spawning_time(
            &self.options.shell,
            self.options.output_style,
            self.options.command_output_policy,
        )?;

        for (num, cmd) in self.commands.iter().enumerate() {
            self.results
                .push(self.run_benchmark(num, cmd, shell_spawning_time)?);

            // We export (all results so far) after each individual benchmark, because
            // we would risk losing all results if a later benchmark fails.
            self.export_manager
                .write_results(&self.results, self.options.time_unit)?;
        }

        Ok(())
    }

    pub fn print_relative_speed_comparison(&self) {
        if self.options.output_style == OutputStyleOption::Disabled {
            return;
        }

        if self.results.len() < 2 {
            return;
        }

        if let Some(mut annotated_results) = relative_speed::compute(&self.results) {
            annotated_results.sort_by(|l, r| relative_speed::compare_mean_time(l.result, r.result));

            let fastest = &annotated_results[0];
            let others = &annotated_results[1..];

            println!("{}", "Summary".bold());
            println!("  '{}' ran", fastest.result.command.cyan());

            for item in others {
                println!(
                    "{}{} times faster than '{}'",
                    format!("{:8.2}", item.relative_speed).bold().green(),
                    if let Some(stddev) = item.relative_speed_stddev {
                        format!(" ± {}", format!("{:.2}", stddev).green())
                    } else {
                        "".into()
                    },
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

    fn run_intermediate_command(
        &self,
        command: &Option<Command<'_>>,
        error_output: &'static str,
    ) -> Result<TimingResult> {
        if let Some(ref cmd) = command {
            let res = time_shell_command(
                &self.options.shell,
                cmd,
                self.options.command_output_policy,
                CmdFailureAction::RaiseError,
                None,
            );
            if res.is_err() {
                bail!(error_output);
            }
            return res.map(|r| r.0);
        }
        Ok(TimingResult {
            ..Default::default()
        })
    }

    /// Run the command specified by `--setup`.
    fn run_setup_command(
        &self,
        parameters: impl IntoIterator<Item = ParameterNameAndValue<'a>>,
    ) -> Result<TimingResult> {
        let command = self
            .options
            .setup_command
            .as_ref()
            .map(|setup_command| Command::new_parametrized(None, setup_command, parameters));

        let error_output = "The setup command terminated with a non-zero exit code. \
                        Append ' || true' to the command if you are sure that this can be ignored.";

        self.run_intermediate_command(&command, error_output)
    }

    /// Run the command specified by `--prepare`.
    fn run_preparation_command(&self, command: &Option<Command<'_>>) -> Result<TimingResult> {
        let error_output = "The preparation command terminated with a non-zero exit code. \
                        Append ' || true' to the command if you are sure that this can be ignored.";

        self.run_intermediate_command(command, error_output)
    }

    /// Run the command specified by `--cleanup`.
    fn run_cleanup_command(&self, command: &Option<Command<'_>>) -> Result<TimingResult> {
        let error_output = "The cleanup command terminated with a non-zero exit code. \
                        Append ' || true' to the command if you are sure that this can be ignored.";

        self.run_intermediate_command(command, error_output)
    }

    /// Run the benchmark for a single shell command
    pub fn run_benchmark(
        &self,
        num: usize,
        cmd: &Command<'_>,
        shell_spawning_time: TimingResult,
    ) -> Result<BenchmarkResult> {
        let command_name = cmd.get_name();
        if self.options.output_style != OutputStyleOption::Disabled {
            println!(
                "{}{}: {}",
                "Benchmark ".bold(),
                (num + 1).to_string().bold(),
                command_name,
            );
        }

        let mut times_real: Vec<Second> = vec![];
        let mut times_user: Vec<Second> = vec![];
        let mut times_system: Vec<Second> = vec![];
        let mut exit_codes: Vec<Option<i32>> = vec![];
        let mut all_succeeded = true;

        // Run init command
        let prepare_cmd = self.options.preparation_command.as_ref().map(|values| {
            let preparation_command = if values.len() == 1 {
                &values[0]
            } else {
                &values[num]
            };
            Command::new_parametrized(
                None,
                preparation_command,
                cmd.get_parameters().iter().cloned(),
            )
        });

        self.run_setup_command(cmd.get_parameters().iter().cloned())?;

        // Warmup phase
        if self.options.warmup_count > 0 {
            let progress_bar = if self.options.output_style != OutputStyleOption::Disabled {
                Some(get_progress_bar(
                    self.options.warmup_count,
                    "Performing warmup runs",
                    self.options.output_style,
                ))
            } else {
                None
            };

            for _ in 0..self.options.warmup_count {
                let _ = self.run_preparation_command(&prepare_cmd)?;
                let _ = time_shell_command(
                    &self.options.shell,
                    cmd,
                    self.options.command_output_policy,
                    self.options.command_failure_action,
                    None,
                )?;
                if let Some(bar) = progress_bar.as_ref() {
                    bar.inc(1)
                }
            }
            if let Some(bar) = progress_bar.as_ref() {
                bar.finish_and_clear()
            }
        }

        // Set up progress bar (and spinner for initial measurement)
        let progress_bar = if self.options.output_style != OutputStyleOption::Disabled {
            Some(get_progress_bar(
                self.options.run_bounds.min,
                "Initial time measurement",
                self.options.output_style,
            ))
        } else {
            None
        };

        let prepare_result = self.run_preparation_command(&prepare_cmd)?;

        // Initial timing run
        let (res, status) = time_shell_command(
            &self.options.shell,
            cmd,
            self.options.command_output_policy,
            self.options.command_failure_action,
            Some(shell_spawning_time),
        )?;
        let success = status.success();

        // Determine number of benchmark runs
        let runs_in_min_time = (self.options.min_benchmarking_time
            / (res.time_real + prepare_result.time_real + shell_spawning_time.time_real))
            as u64;

        let count = {
            let min = cmp::max(runs_in_min_time, self.options.run_bounds.min);

            self.options
                .run_bounds
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
        exit_codes.push(extract_exit_code(status));

        all_succeeded = all_succeeded && success;

        // Re-configure the progress bar
        if let Some(bar) = progress_bar.as_ref() {
            bar.set_length(count)
        }
        if let Some(bar) = progress_bar.as_ref() {
            bar.inc(1)
        }

        // Gather statistics
        for _ in 0..count_remaining {
            self.run_preparation_command(&prepare_cmd)?;

            let msg = {
                let mean = format_duration(mean(&times_real), self.options.time_unit);
                format!("Current estimate: {}", mean.to_string().green())
            };

            if let Some(bar) = progress_bar.as_ref() {
                bar.set_message(msg.to_owned())
            }

            let (res, status) = time_shell_command(
                &self.options.shell,
                cmd,
                self.options.command_output_policy,
                self.options.command_failure_action,
                Some(shell_spawning_time),
            )?;
            let success = status.success();

            times_real.push(res.time_real);
            times_user.push(res.time_user);
            times_system.push(res.time_system);
            exit_codes.push(extract_exit_code(status));

            all_succeeded = all_succeeded && success;

            if let Some(bar) = progress_bar.as_ref() {
                bar.inc(1)
            }
        }

        if let Some(bar) = progress_bar.as_ref() {
            bar.finish_and_clear()
        }

        // Compute statistical quantities
        let t_num = times_real.len();
        let t_mean = mean(&times_real);
        let t_stddev = if times_real.len() > 1 {
            Some(standard_deviation(&times_real, Some(t_mean)))
        } else {
            None
        };
        let t_median = median(&times_real);
        let t_min = min(&times_real);
        let t_max = max(&times_real);

        let user_mean = mean(&times_user);
        let system_mean = mean(&times_system);

        // Formatting and console output
        let (mean_str, time_unit) = format_duration_unit(t_mean, self.options.time_unit);
        let min_str = format_duration(t_min, Some(time_unit));
        let max_str = format_duration(t_max, Some(time_unit));
        let num_str = format!("{} runs", t_num);

        let user_str = format_duration(user_mean, Some(time_unit));
        let system_str = format_duration(system_mean, Some(time_unit));

        if self.options.output_style != OutputStyleOption::Disabled {
            if times_real.len() == 1 {
                println!(
                    "  Time ({} ≡):        {:>8}  {:>8}     [User: {}, System: {}]",
                    "abs".green().bold(),
                    mean_str.green().bold(),
                    "        ".to_string(), // alignment
                    user_str.blue(),
                    system_str.blue()
                );
            } else {
                let stddev_str = format_duration(t_stddev.unwrap(), Some(time_unit));

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
                    "  Range ({} … {}):   {:>8} … {:>8}    {}",
                    "min".cyan(),
                    "max".purple(),
                    min_str.cyan(),
                    max_str.purple(),
                    num_str.dimmed()
                );
            }
        }

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
        } else if scores.iter().any(|&s| s.abs() > OUTLIER_THRESHOLD) {
            warnings.push(Warnings::OutliersDetected);
        }

        if !warnings.is_empty() {
            eprintln!(" ");

            for warning in &warnings {
                eprintln!("  {}: {}", "Warning".yellow(), warning);
            }
        }

        if self.options.output_style != OutputStyleOption::Disabled {
            println!(" ");
        }

        // Run cleanup command
        let cleanup_cmd = self
            .options
            .cleanup_command
            .as_ref()
            .map(|cleanup_command| {
                Command::new_parametrized(
                    None,
                    cleanup_command,
                    cmd.get_parameters().iter().cloned(),
                )
            });
        self.run_cleanup_command(&cleanup_cmd)?;

        Ok(BenchmarkResult {
            command: command_name,
            mean: t_mean,
            stddev: t_stddev,
            median: t_median,
            user: user_mean,
            system: system_mean,
            min: t_min,
            max: t_max,
            times: Some(times_real),
            exit_codes,
            parameters: cmd
                .get_parameters()
                .iter()
                .map(|(name, value)| ((*name).to_string(), value.to_string()))
                .collect(),
        })
    }
}
