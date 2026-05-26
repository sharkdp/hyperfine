pub mod benchmark_result;
pub mod executor;
pub mod relative_speed;
pub mod scheduler;
pub mod sequential;
pub mod timing_result;

use std::cmp;

use crate::benchmark::executor::BenchmarkIteration;
use crate::command::Command;
use crate::options::{
    CmdFailureAction, CommandOutputPolicy, ExecutorKind, Options, OutputStyleOption,
};
use crate::outlier_detection::{modified_zscores, OUTLIER_THRESHOLD};
use crate::output::format::{format_duration, format_duration_unit};
use crate::output::progress_bar::get_progress_bar;
use crate::output::warnings::{OutlierWarningOptions, Warnings};
use crate::parameter::ParameterNameAndValue;
use crate::util::exit_code::extract_exit_code;
use crate::util::min_max::{max, min};
use crate::util::units::Second;
use benchmark_result::BenchmarkResult;
use timing_result::TimingResult;

use anyhow::{anyhow, Result};
use colored::*;
use statistical::{mean, median, standard_deviation};

use self::executor::Executor;

/// Threshold for warning about fast execution time
pub const MIN_EXECUTION_TIME: Second = 5e-3;

pub(crate) struct BenchmarkState {
    times_real: Vec<Second>,
    times_user: Vec<Second>,
    times_system: Vec<Second>,
    memory_usage_byte: Vec<u64>,
    exit_codes: Vec<Option<i32>>,
    all_succeeded: bool,
}

impl BenchmarkState {
    pub(crate) fn new() -> Self {
        Self {
            times_real: vec![],
            times_user: vec![],
            times_system: vec![],
            memory_usage_byte: vec![],
            exit_codes: vec![],
            all_succeeded: true,
        }
    }
}

pub(crate) struct IterationOverheads {
    pub preparation: Second,
    pub conclusion: Second,
}

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
    ) -> Result<TimingResult> {
        self.executor
            .run_command_and_measure(
                command,
                executor::BenchmarkIteration::NonBenchmarkRun,
                Some(CmdFailureAction::RaiseError),
                output_policy,
            )
            .map(|r| r.0)
            .map_err(|_| anyhow!(error_output))
    }

    /// Run the command specified by `--setup`.
    fn run_setup_command(
        &self,
        parameters: impl IntoIterator<Item = ParameterNameAndValue<'a>>,
        output_policy: &CommandOutputPolicy,
    ) -> Result<TimingResult> {
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
    ) -> Result<TimingResult> {
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

    /// Print the benchmark header line.
    pub(crate) fn print_header(&self) {
        if self.options.output_style != OutputStyleOption::Disabled {
            println!(
                "{}{}: {}",
                "Benchmark ".bold(),
                (self.number + 1).to_string().bold(),
                self.command.get_name_with_unused_parameters(),
            );
        }
    }

    fn output_policy(&self) -> &CommandOutputPolicy {
        &self.options.command_output_policies[self.number]
    }

    fn preparation_command(&self) -> Option<Command<'a>> {
        self.options.preparation_command.as_ref().map(|values| {
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
        })
    }

    fn conclusion_command(&self) -> Option<Command<'a>> {
        self.options.conclusion_command.as_ref().map(|values| {
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
        })
    }

    fn run_preparation_command(&self) -> Result<Option<TimingResult>> {
        self.preparation_command()
            .as_ref()
            .map(|cmd| self.run_preparation_command_impl(cmd, self.output_policy()))
            .transpose()
    }

    fn run_conclusion_command(&self) -> Result<Option<TimingResult>> {
        self.conclusion_command()
            .as_ref()
            .map(|cmd| self.run_conclusion_command_impl(cmd, self.output_policy()))
            .transpose()
    }

    /// Run setup for this benchmark.
    pub(crate) fn run_setup(&self) -> Result<()> {
        self.run_setup_command(
            self.command.get_parameters().iter().cloned(),
            self.output_policy(),
        )?;
        Ok(())
    }

    /// Run warmup iterations for this benchmark.
    pub(crate) fn run_warmup(&self) -> Result<()> {
        if self.options.warmup_count == 0 {
            return Ok(());
        }

        let output_policy = self.output_policy();
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
            let _ = self.run_preparation_command()?;
            let _ = self.executor.run_command_and_measure(
                self.command,
                BenchmarkIteration::Warmup(i),
                None,
                output_policy,
            )?;
            let _ = self.run_conclusion_command()?;
            if let Some(bar) = progress_bar.as_ref() {
                bar.inc(1)
            }
        }
        if let Some(bar) = progress_bar.as_ref() {
            bar.finish_and_clear()
        }

        Ok(())
    }

    /// Run a single measured benchmark iteration and append the result to `state`.
    pub(crate) fn run_measured_iteration(
        &self,
        iteration: u64,
        state: &mut BenchmarkState,
    ) -> Result<IterationOverheads> {
        let output_policy = self.output_policy();

        let preparation_result = self.run_preparation_command()?;
        let preparation_overhead =
            preparation_result.map_or(0.0, |res| res.time_real + self.executor.time_overhead());

        let (res, status) = self.executor.run_command_and_measure(
            self.command,
            BenchmarkIteration::Benchmark(iteration),
            None,
            output_policy,
        )?;
        let success = status.success();

        let conclusion_result = self.run_conclusion_command()?;
        let conclusion_overhead =
            conclusion_result.map_or(0.0, |res| res.time_real + self.executor.time_overhead());

        state.times_real.push(res.time_real);
        state.times_user.push(res.time_user);
        state.times_system.push(res.time_system);
        state.memory_usage_byte.push(res.memory_usage_byte);
        state.exit_codes.push(extract_exit_code(status));
        state.all_succeeded = state.all_succeeded && success;

        Ok(IterationOverheads {
            preparation: preparation_overhead,
            conclusion: conclusion_overhead,
        })
    }

    /// Determine how many benchmark runs should be performed, based on the first measurement.
    pub(crate) fn compute_run_count(
        &self,
        first_run_time: Second,
        overheads: IterationOverheads,
    ) -> u64 {
        let runs_in_min_time = (self.options.min_benchmarking_time
            / (first_run_time
                + self.executor.time_overhead()
                + overheads.preparation
                + overheads.conclusion)) as u64;

        let min = cmp::max(runs_in_min_time, self.options.run_bounds.min);

        self.options
            .run_bounds
            .max
            .as_ref()
            .map(|max| cmp::min(min, *max))
            .unwrap_or(min)
    }

    /// Print statistics, warnings, run cleanup, and build the final result.
    pub(crate) fn finalize(&self, state: BenchmarkState) -> Result<BenchmarkResult> {
        let BenchmarkState {
            times_real,
            times_user,
            times_system,
            memory_usage_byte,
            exit_codes,
            all_succeeded,
        } = state;

        self.print_statistics(&times_real, &times_user, &times_system);
        self.print_warnings(&times_real, all_succeeded);

        if self.options.output_style != OutputStyleOption::Disabled {
            println!(" ");
        }

        self.run_cleanup_command(
            self.command.get_parameters().iter().cloned(),
            self.output_policy(),
        )?;

        Ok(self.build_result(
            times_real,
            times_user,
            times_system,
            memory_usage_byte,
            exit_codes,
        ))
    }

    fn print_statistics(
        &self,
        times_real: &[Second],
        times_user: &[Second],
        times_system: &[Second],
    ) {
        if self.options.output_style == OutputStyleOption::Disabled {
            return;
        }

        let t_num = times_real.len();
        let t_mean = mean(times_real);
        let t_stddev = if times_real.len() > 1 {
            Some(standard_deviation(times_real, Some(t_mean)))
        } else {
            None
        };
        let t_min = min(times_real);
        let t_max = max(times_real);

        let user_mean = mean(times_user);
        let system_mean = mean(times_system);

        let (mean_str, time_unit) = format_duration_unit(t_mean, self.options.time_unit);
        let min_str = format_duration(t_min, Some(time_unit));
        let max_str = format_duration(t_max, Some(time_unit));
        let num_str = format!("{t_num} runs");

        let user_str = format_duration(user_mean, Some(time_unit));
        let system_str = format_duration(system_mean, Some(time_unit));

        if times_real.len() == 1 {
            println!(
                "  Time ({} ≡):        {:>8}  {:>8}     [User: {}, System: {}]",
                "abs".green().bold(),
                mean_str.green().bold(),
                "        ",
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

    fn print_warnings(&self, times_real: &[Second], all_succeeded: bool) {
        let mut warnings = vec![];

        if matches!(self.options.executor_kind, ExecutorKind::Shell(_))
            && times_real.iter().any(|&t| t < MIN_EXECUTION_TIME)
        {
            warnings.push(Warnings::FastExecutionTime);
        }

        if !all_succeeded {
            warnings.push(Warnings::NonZeroExitCode);
        }

        let scores = modified_zscores(times_real);

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
                times_real[0],
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
    }

    fn build_result(
        &self,
        times_real: Vec<Second>,
        times_user: Vec<Second>,
        times_system: Vec<Second>,
        memory_usage_byte: Vec<u64>,
        exit_codes: Vec<Option<i32>>,
    ) -> BenchmarkResult {
        let t_mean = mean(&times_real);
        let t_stddev = if times_real.len() > 1 {
            Some(standard_deviation(&times_real, Some(t_mean)))
        } else {
            None
        };

        BenchmarkResult {
            command: self.command.get_name(),
            command_with_unused_parameters: self.command.get_name_with_unused_parameters(),
            mean: t_mean,
            stddev: t_stddev,
            median: median(&times_real),
            user: mean(&times_user),
            system: mean(&times_system),
            min: min(&times_real),
            max: max(&times_real),
            times: Some(times_real),
            memory_usage_byte: Some(memory_usage_byte),
            exit_codes,
            parameters: self
                .command
                .get_parameters()
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
        }
    }

    /// Run the command specified by `--prepare`.
    fn run_preparation_command_impl(
        &self,
        command: &Command<'_>,
        output_policy: &CommandOutputPolicy,
    ) -> Result<TimingResult> {
        let error_output = "The preparation command terminated with a non-zero exit code. \
                            Append ' || true' to the command if you are sure that this can be ignored.";

        self.run_intermediate_command(command, error_output, output_policy)
    }

    /// Run the command specified by `--conclude`.
    fn run_conclusion_command_impl(
        &self,
        command: &Command<'_>,
        output_policy: &CommandOutputPolicy,
    ) -> Result<TimingResult> {
        let error_output = "The conclusion command terminated with a non-zero exit code. \
                            Append ' || true' to the command if you are sure that this can be ignored.";

        self.run_intermediate_command(command, error_output, output_policy)
    }

    /// Run the benchmark for a single command
    pub fn run(&self) -> Result<BenchmarkResult> {
        self.print_header();

        let mut state = BenchmarkState::new();

        self.run_setup()?;
        self.run_warmup()?;

        let progress_bar = if self.options.output_style != OutputStyleOption::Disabled {
            Some(get_progress_bar(
                self.options.run_bounds.min,
                "Initial time measurement",
                self.options.output_style,
            ))
        } else {
            None
        };

        let overheads = self.run_measured_iteration(0, &mut state)?;
        let count = self.compute_run_count(state.times_real[0], overheads);
        let count_remaining = count - 1;

        if let Some(bar) = progress_bar.as_ref() {
            bar.set_length(count)
        }
        if let Some(bar) = progress_bar.as_ref() {
            bar.inc(1)
        }

        for i in 0..count_remaining {
            let msg = {
                let mean = format_duration(mean(&state.times_real), self.options.time_unit);
                format!("Current estimate: {}", mean.to_string().green())
            };

            if let Some(bar) = progress_bar.as_ref() {
                bar.set_message(msg.to_owned())
            }

            self.run_measured_iteration(i + 1, &mut state)?;

            if let Some(bar) = progress_bar.as_ref() {
                bar.inc(1)
            }
        }

        if let Some(bar) = progress_bar.as_ref() {
            bar.finish_and_clear()
        }

        self.finalize(state)
    }
}
