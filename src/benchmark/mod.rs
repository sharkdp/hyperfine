pub mod benchmark_result;
pub mod executor;
pub mod measurement;
pub mod relative_speed;
pub mod scheduler;

use std::cmp;

use crate::benchmark::benchmark_result::Parameter;
use crate::benchmark::executor::BenchmarkIteration;
use crate::benchmark::measurement::{Measurement, Measurements};
use crate::command::Command;
use crate::options::{
    CmdFailureAction, CommandOutputPolicy, ExecutorKind, Options, OutputStyleOption,
};
use crate::outlier_detection::OUTLIER_THRESHOLD;
use crate::output::progress_bar::get_progress_bar;
use crate::output::warnings::{OutlierWarningOptions, Warnings};
use crate::parameter::ParameterNameAndValue;
use crate::quantity::{self, const_time_from_seconds, Time, TimeQuantity};
use benchmark_result::BenchmarkResult;

use anyhow::{anyhow, Result};
use colored::*;

use self::executor::Executor;

/// Threshold for warning about fast execution time
pub const MIN_EXECUTION_TIME: Time = const_time_from_seconds(0.005);

pub struct Benchmark<'a> {
    number: usize,
    command: &'a Command<'a>,
    options: &'a Options,
    executor: &'a dyn Executor,
}

impl<'a> Benchmark<'a> {
    pub fn new(
        number: usize,
        command: &'a Command<'a>,
        options: &'a Options,
        executor: &'a dyn Executor,
    ) -> Self {
        Benchmark {
            number,
            command,
            options,
            executor,
        }
    }

    /// Run setup, cleanup, or preparation commands
    fn run_intermediate_command(
        &self,
        command: &Command<'_>,
        error_output: &'static str,
        output_policy: &CommandOutputPolicy,
    ) -> Result<Measurement> {
        self.executor
            .run_command_and_measure(
                command,
                executor::BenchmarkIteration::NonBenchmarkRun,
                Some(CmdFailureAction::RaiseError),
                output_policy,
            )
            .map_err(|_| anyhow!(error_output))
    }

    /// Run the command specified by `--setup`.
    fn run_setup_command(
        &self,
        parameters: impl IntoIterator<Item = ParameterNameAndValue<'a>>,
        output_policy: &CommandOutputPolicy,
    ) -> Result<Measurement> {
        let command = self
            .options
            .setup_command
            .as_ref()
            .map(|setup_command| Command::new_parametrized(None, setup_command, parameters));

        let error_output = "The setup command terminated with a non-zero exit code. \
                            Append ' || true' to the command if you are sure that this can be ignored.";

        Ok(command
            .map(|cmd| self.run_intermediate_command(&cmd, error_output, output_policy))
            .transpose()?
            .unwrap_or_default())
    }

    /// Run the command specified by `--cleanup`.
    fn run_cleanup_command(
        &self,
        parameters: impl IntoIterator<Item = ParameterNameAndValue<'a>>,
        output_policy: &CommandOutputPolicy,
    ) -> Result<Measurement> {
        let command = self
            .options
            .cleanup_command
            .as_ref()
            .map(|cleanup_command| Command::new_parametrized(None, cleanup_command, parameters));

        let error_output = "The cleanup command terminated with a non-zero exit code. \
                            Append ' || true' to the command if you are sure that this can be ignored.";

        Ok(command
            .map(|cmd| self.run_intermediate_command(&cmd, error_output, output_policy))
            .transpose()?
            .unwrap_or_default())
    }

    /// Run the command specified by `--prepare`.
    fn run_preparation_command(
        &self,
        command: &Command<'_>,
        output_policy: &CommandOutputPolicy,
    ) -> Result<Measurement> {
        let error_output = "The preparation command terminated with a non-zero exit code. \
                            Append ' || true' to the command if you are sure that this can be ignored.";

        self.run_intermediate_command(command, error_output, output_policy)
    }

    /// Run the command specified by `--conclude`.
    fn run_conclusion_command(
        &self,
        command: &Command<'_>,
        output_policy: &CommandOutputPolicy,
    ) -> Result<Measurement> {
        let error_output = "The conclusion command terminated with a non-zero exit code. \
                            Append ' || true' to the command if you are sure that this can be ignored.";

        self.run_intermediate_command(command, error_output, output_policy)
    }

    /// Run the benchmark for a single command
    pub fn run(&self) -> Result<BenchmarkResult> {
        if self.options.output_style != OutputStyleOption::Disabled {
            println!(
                "{}{}: {}",
                "Benchmark ".bold(),
                (self.number + 1).to_string().bold(),
                self.command.get_name_with_unused_parameters(),
            );
        }

        let mut measurements = Measurements::default();
        let mut all_succeeded = true;

        let output_policy = &self.options.command_output_policies[self.number];

        let preparation_command = self.options.preparation_command.as_ref().map(|values| {
            let preparation_command = if values.len() == 1 {
                &values[0]
            } else {
                &values[self.number]
            };
            Command::new_parametrized(
                None,
                preparation_command,
                self.command.get_parameters().iter().cloned(),
            )
        });

        let run_preparation_command = || {
            preparation_command
                .as_ref()
                .map(|cmd| self.run_preparation_command(cmd, output_policy))
                .transpose()
        };

        let conclusion_command = self.options.conclusion_command.as_ref().map(|values| {
            let conclusion_command = if values.len() == 1 {
                &values[0]
            } else {
                &values[self.number]
            };
            Command::new_parametrized(
                None,
                conclusion_command,
                self.command.get_parameters().iter().cloned(),
            )
        });
        let run_conclusion_command = || {
            conclusion_command
                .as_ref()
                .map(|cmd| self.run_conclusion_command(cmd, output_policy))
                .transpose()
        };

        self.run_setup_command(self.command.get_parameters().iter().cloned(), output_policy)?;

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

            for i in 0..self.options.warmup_count {
                let _ = run_preparation_command()?;
                let _ = self.executor.run_command_and_measure(
                    self.command,
                    BenchmarkIteration::Warmup(i),
                    None,
                    output_policy,
                )?;
                let _ = run_conclusion_command()?;
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

        let preparation_result = run_preparation_command()?;
        let preparation_overhead = preparation_result.map_or(Time::zero(), |res| {
            res.time_wall_clock + self.executor.time_overhead()
        });

        // Initial timing run
        let measurement = self.executor.run_command_and_measure(
            self.command,
            BenchmarkIteration::Benchmark(0),
            None,
            output_policy,
        )?;
        let success = measurement.exit_status.success();

        let conclusion_result = run_conclusion_command()?;
        let conclusion_overhead = conclusion_result.map_or(Time::zero(), |res| {
            res.time_wall_clock + self.executor.time_overhead()
        });

        // Determine number of benchmark runs
        let runs_in_min_time = (self.options.min_benchmarking_time
            / (measurement.time_wall_clock
                + self.executor.time_overhead()
                + preparation_overhead
                + conclusion_overhead))
            .get::<quantity::ratio>() as u64;

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
        measurements.push(measurement);

        all_succeeded = all_succeeded && success;

        // Re-configure the progress bar
        if let Some(bar) = progress_bar.as_ref() {
            bar.set_length(count)
        }
        if let Some(bar) = progress_bar.as_ref() {
            bar.inc(1)
        }

        // Gather statistics (perform the actual benchmark)
        for i in 0..count_remaining {
            run_preparation_command()?;

            let msg = {
                let t_wall_clock_mean = measurements.time_wall_clock_mean();
                let time_unit = t_wall_clock_mean.suitable_unit();
                let mean = t_wall_clock_mean.format(time_unit);
                format!("Current estimate: {}", mean.to_string().green())
            };

            if let Some(bar) = progress_bar.as_ref() {
                bar.set_message(msg.to_owned())
            }

            let measurement = self.executor.run_command_and_measure(
                self.command,
                BenchmarkIteration::Benchmark(i + 1),
                None,
                output_policy,
            )?;
            let success = measurement.exit_status.success();
            measurements.push(measurement);

            all_succeeded = all_succeeded && success;

            if let Some(bar) = progress_bar.as_ref() {
                bar.inc(1)
            }

            run_conclusion_command()?;
        }

        if let Some(bar) = progress_bar.as_ref() {
            bar.finish_and_clear()
        }

        // Formatting and console output
        let t_wall_clock_mean = measurements.time_wall_clock_mean();
        let time_unit = self
            .options
            .time_unit
            .unwrap_or(t_wall_clock_mean.suitable_unit());
        let mean_str = t_wall_clock_mean.format(time_unit);
        let min_str = measurements.min().format(time_unit);
        let max_str = measurements.max().format(time_unit);
        let num_str = format!("{num_runs} runs", num_runs = measurements.len());

        let user_str = measurements.time_user_mean().format(time_unit);
        let system_str = measurements.time_system_mean().format(time_unit);

        if self.options.output_style != OutputStyleOption::Disabled {
            if measurements.len() == 1 {
                println!(
                    "  Time ({} ≡):        {:>8}  {:>8}     [User: {}, System: {}]",
                    "abs".green().bold(),
                    mean_str.green().bold(),
                    "        ", // alignment
                    user_str.blue(),
                    system_str.blue()
                );
            } else {
                let stddev_str = measurements.stddev().unwrap().format(time_unit);

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
        if matches!(self.options.executor_kind, ExecutorKind::Shell(_))
            && measurements
                .wall_clock_times()
                .iter()
                .any(|&t| t < MIN_EXECUTION_TIME)
        {
            warnings.push(Warnings::FastExecutionTime);
        }

        // Check program exit codes
        if !all_succeeded {
            warnings.push(Warnings::NonZeroExitCode);
        }

        // Run outlier detection
        let scores = measurements.modified_zscores();

        let outlier_warning_options = OutlierWarningOptions {
            warmup_in_use: self.options.warmup_count > 0,
            prepare_in_use: self
                .options
                .preparation_command
                .as_ref()
                .map(|v| v.len())
                .unwrap_or(0)
                > 0,
        };

        if scores[0] > OUTLIER_THRESHOLD {
            warnings.push(Warnings::SlowInitialRun(
                measurements.wall_clock_times()[0],
                outlier_warning_options,
            ));
        } else if scores.iter().any(|&s| s.abs() > OUTLIER_THRESHOLD) {
            warnings.push(Warnings::OutliersDetected(outlier_warning_options));
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

        self.run_cleanup_command(self.command.get_parameters().iter().cloned(), output_policy)?;

        Ok(BenchmarkResult {
            command: self.command.get_name(),
            measurements,
            parameters: self
                .command
                .get_parameters()
                .iter()
                .map(|(name, value)| {
                    (
                        name.to_string(),
                        Parameter {
                            value: value.to_string(),
                            is_unused: self.command.is_parameter_unused(name),
                        },
                    )
                })
                .collect(),
        })
    }
}
