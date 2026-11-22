pub mod benchmark_result;
pub mod executor;
pub mod measurement;
pub mod scheduler;

use std::cmp;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::benchmark::benchmark_result::Parameter;
use crate::benchmark::executor::BenchmarkIteration;
use crate::benchmark::measurement::{Measurement, Measurements};
use crate::command::Command;
use crate::options::{CommandOutputPolicy, ExecutorKind, Options, OutputStyleOption};
use crate::outlier_detection::OUTLIER_THRESHOLD;
use crate::output::console_writeln;
use crate::output::progress_bar::{
    finish_initial_measurement, get_progress_bar, set_benchmark_header, start_initial_measurement,
};
use crate::output::report;
use crate::output::warnings::{OutlierWarningOptions, Warnings};
use crate::parameter::ParameterNameAndValue;
use crate::quantity::{self, const_time_from_seconds, Time, Zero};
use benchmark_result::BenchmarkResult;

use anyhow::{anyhow, ensure, Result};
use colored::*;

use self::executor::Executor;

/// Threshold for warning about fast execution time
pub const MIN_EXECUTION_TIME: Time = const_time_from_seconds(0.005);

/// How often the run timer updates the elapsed time of the benchmark line in progress. It runs
/// faster than the bar redraws, so that the first redraw of a line already knows that it started.
const RUN_TIMER_INTERVAL: Duration = Duration::from_millis(50);

/// Keeps the time that the benchmark line in progress has taken up to date. The executor counts
/// the starts and the ends of lines in a separate counter, and the timer starts a new count of
/// the line time when a line starts.
struct RunTimer {
    stop: Arc<AtomicBool>,
}

impl RunTimer {
    fn start(current_run: Arc<AtomicU64>, lines: Arc<AtomicU64>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let should_stop = Arc::clone(&stop);
        thread::spawn(move || {
            let mut line_started = Instant::now();
            let mut seen_lines = lines.load(Ordering::Relaxed);
            while !should_stop.load(Ordering::Relaxed) {
                thread::sleep(RUN_TIMER_INTERVAL);
                let now = Instant::now();
                let lines_now = lines.load(Ordering::Relaxed);
                if lines_now != seen_lines {
                    line_started = now;
                    seen_lines = lines_now;
                    current_run.store(0, Ordering::Relaxed);
                    continue;
                }
                current_run.store(
                    now.duration_since(line_started).as_millis() as u64,
                    Ordering::Relaxed,
                );
            }
        });
        RunTimer { stop }
    }
}

impl Drop for RunTimer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

pub struct Benchmark<'a> {
    number: usize,
    command: &'a Command<'a>,
    options: &'a Options,
    executor: &'a dyn Executor,
    current_run: Arc<AtomicU64>,
    lines: Arc<AtomicU64>,
}

impl<'a> Benchmark<'a> {
    pub fn new(
        number: usize,
        command: &'a Command<'a>,
        options: &'a Options,
        executor: &'a dyn Executor,
        current_run: Arc<AtomicU64>,
        lines: Arc<AtomicU64>,
    ) -> Self {
        Benchmark {
            number,
            command,
            options,
            executor,
            current_run,
            lines,
        }
    }

    /// Run setup, cleanup, preparation, or conclusion commands
    fn run_intermediate_command(
        &self,
        command: &Command<'_>,
        kind: &'static str,
        output_policy: &CommandOutputPolicy,
        iteration: executor::BenchmarkIteration,
    ) -> Result<Measurement> {
        let command = command.clone().with_environment_from(self.command);
        self.executor
            .run_intermediate_command_and_measure(&command, iteration, output_policy)
            .map_err(|error| {
                let hint = if cfg!(windows) {
                    "Append ' || exit /b 0' to the command if this failure can be ignored."
                } else {
                    "Append ' || true' to the command if this failure can be ignored."
                };
                anyhow!("The {kind} command failed: {error:#} {hint}")
            })
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

        Ok(command
            .map(|cmd| {
                self.run_intermediate_command(
                    &cmd,
                    "setup",
                    output_policy,
                    BenchmarkIteration::NonBenchmarkRun,
                )
            })
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

        Ok(command
            .map(|cmd| {
                self.run_intermediate_command(
                    &cmd,
                    "cleanup",
                    output_policy,
                    BenchmarkIteration::NonBenchmarkRun,
                )
            })
            .transpose()?
            .unwrap_or_default())
    }

    /// Run the command specified by `--prepare`.
    fn run_preparation_command(
        &self,
        command: &Command<'_>,
        output_policy: &CommandOutputPolicy,
        iteration: executor::BenchmarkIteration,
    ) -> Result<Measurement> {
        self.run_intermediate_command(command, "preparation", output_policy, iteration)
    }

    /// Run the command specified by `--conclude`.
    fn run_conclusion_command(
        &self,
        command: &Command<'_>,
        output_policy: &CommandOutputPolicy,
        iteration: executor::BenchmarkIteration,
    ) -> Result<Measurement> {
        self.run_intermediate_command(command, "conclusion", output_policy, iteration)
    }

    fn validate_measurement(&self, measurement: &Measurement) -> Result<()> {
        if self.options.skip_unavailable_metrics {
            return Ok(());
        }
        for selection in &self.options.metrics {
            ensure!(selection.metric.value(measurement).is_some(),
                "Metric '{}' is unavailable for '{}'. Check platform support and hardware-counter permissions.",
                selection.metric.name(), self.command.get_command_line());
        }
        Ok(())
    }

    /// Run the benchmark for a single command
    pub fn run(&self, reference: Option<&BenchmarkResult>) -> Result<BenchmarkResult> {
        let mut measurements = Measurements::default();
        // The timer keeps the run time current while a single benchmark line takes a while
        let _run_timer = RunTimer::start(Arc::clone(&self.current_run), Arc::clone(&self.lines));

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

        let run_preparation_command = |iteration: executor::BenchmarkIteration| {
            preparation_command
                .as_ref()
                .map(|cmd| self.run_preparation_command(cmd, output_policy, iteration))
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
        let run_conclusion_command = |iteration: executor::BenchmarkIteration| {
            conclusion_command
                .as_ref()
                .map(|cmd| self.run_conclusion_command(cmd, output_policy, iteration))
                .transpose()
        };

        self.run_setup_command(self.command.get_parameters().iter().cloned(), output_policy)?;

        let progress_header = format!(
            "{}: {}",
            format!("Benchmark {}", self.number + 1).bold(),
            self.command
                .get_name_with_unused_parameters()
                .white()
                .bold()
        );

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

            if let Some(bar) = &progress_bar {
                set_benchmark_header(bar, progress_header.clone());
            }

            for i in 0..self.options.warmup_count {
                let warmup_iteration = BenchmarkIteration::Warmup(i);
                let _ = run_preparation_command(warmup_iteration)?;
                let _ = self.executor.run_command_and_measure(
                    self.command,
                    warmup_iteration,
                    None,
                    output_policy,
                )?;
                let _ = run_conclusion_command(warmup_iteration)?;
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
                if preparation_command.is_some() {
                    "Running preparation command"
                } else {
                    "Initial run"
                },
                self.options.output_style,
            ))
        } else {
            None
        };

        if let Some(bar) = &progress_bar {
            set_benchmark_header(bar, progress_header);
        }

        let benchmark_iteration = BenchmarkIteration::Benchmark(0);
        let preparation_result = run_preparation_command(benchmark_iteration)?;
        let preparation_overhead = preparation_result.map_or(Time::zero(), |res| {
            res.time_wall_clock + self.executor.time_overhead()
        });

        // Initial timing run
        let initial_run_started = Instant::now();
        if let Some(bar) = progress_bar.as_ref() {
            start_initial_measurement(bar, initial_run_started, Arc::clone(&self.current_run));
        }
        self.executor.start_line();
        let measurement = self.executor.run_command_and_measure(
            self.command,
            benchmark_iteration,
            None,
            output_policy,
        );
        self.executor.end_line();
        let measurement = measurement?;
        self.validate_measurement(&measurement)?;
        let mut all_succeeded = measurement.exit_status.success();

        if let Some(bar) = progress_bar.as_ref() {
            let primary = self.options.metrics[0];
            let value = primary.metric.value(&measurement).unwrap();
            let estimate = primary.display_unit(value).format(value);
            finish_initial_measurement(bar, format!("Current estimate: {}", estimate.green()));
        }

        let conclusion_result = run_conclusion_command(benchmark_iteration)?;
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
        let primary = self.options.metrics[0];
        let mut primary_total = primary.metric.value(&measurement).unwrap();
        measurements.push(measurement);

        // Re-configure the progress bar
        if let Some(bar) = progress_bar.as_ref() {
            bar.set_length(count);
            bar.inc(1);
        }

        // Gather statistics (perform the actual benchmark)
        for i in 0..count_remaining {
            let benchmark_iteration = BenchmarkIteration::Benchmark(i + 1);

            if let Some(bar) = progress_bar.as_ref() {
                let mean = primary_total / measurements.len() as f64;
                bar.set_message(format!(
                    "Current estimate: {}",
                    primary.display_unit(mean).format(mean).green()
                ));
            }

            run_preparation_command(benchmark_iteration)?;

            self.executor.start_line();
            let measurement = self.executor.run_command_and_measure(
                self.command,
                benchmark_iteration,
                None,
                output_policy,
            );
            self.executor.end_line();
            let measurement = measurement?;
            self.validate_measurement(&measurement)?;
            let success = measurement.exit_status.success();
            primary_total += primary.metric.value(&measurement).unwrap();
            measurements.push(measurement);

            all_succeeded = all_succeeded && success;

            if let Some(bar) = progress_bar.as_ref() {
                bar.inc(1)
            }

            run_conclusion_command(benchmark_iteration)?;
        }

        if let Some(bar) = progress_bar.as_ref() {
            bar.finish_and_clear()
        }

        if self.options.output_style != OutputStyleOption::Disabled {
            report::print(
                self.number,
                &self.command.get_name_with_unused_parameters(),
                &measurements,
                &self.options.metrics,
                self.options.skip_unavailable_metrics,
                reference,
            )?;
        }

        // Warnings
        let mut warnings = vec![];

        // Check execution time
        if self
            .options
            .metrics
            .iter()
            .any(|m| m.metric == crate::metric::Metric::TimeWallClock)
            && matches!(self.options.executor_kind, ExecutorKind::Shell(_))
            && measurements
                .wall_clock_times()
                .any(|t| t < MIN_EXECUTION_TIME)
        {
            warnings.push(Warnings::FastExecutionTime);
        }

        // Check program exit codes
        if !all_succeeded {
            warnings.push(Warnings::NonZeroExitCode);
        }

        // Run outlier detection
        let scores = measurements.modified_zscores(primary.metric);

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
            warnings.push(Warnings::InitialRunOutlier(
                primary,
                primary.metric.value(&measurements.measurements[0]).unwrap(),
                outlier_warning_options,
            ));
        }

        if !warnings.is_empty() {
            let mut stderr = io::stderr().lock();
            console_writeln!(stderr, " ")?;

            for warning in &warnings {
                console_writeln!(stderr, "  {}: {}", "Warning".yellow(), warning)?;
            }
        }

        if self.options.output_style != OutputStyleOption::Disabled {
            console_writeln!(io::stdout(), " ")?;
        }

        self.run_cleanup_command(self.command.get_parameters().iter().cloned(), output_policy)?;

        let command = self.command.get_command_line();
        let name = self.command.get_name();
        let name = (name != command).then_some(name);

        Ok(BenchmarkResult {
            command,
            name,
            environment: self
                .command
                .get_environment()
                .map(|(name, value)| (name.to_owned(), value))
                .collect(),
            display_name: self.command.get_name_with_unused_parameters(),
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
                        },
                    )
                })
                .collect(),
        })
    }
}
