use super::benchmark_result::BenchmarkResult;
use super::executor::{Executor, MockExecutor, RawExecutor, ShellExecutor};
use super::{Benchmark, BenchmarkRun};

use crate::command::Commands;
use crate::export::ExportManager;
use crate::options::{ExecutorKind, Options, OutputStyleOption};
use crate::output::progress_bar::{get_progress_bar, set_benchmark_header};

use anyhow::Result;

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
        let mut executor: Box<dyn Executor> = match self.options.executor_kind {
            ExecutorKind::Raw => Box::new(RawExecutor::new(self.options)),
            ExecutorKind::Mock(ref shell) => Box::new(MockExecutor::new(shell.clone())),
            ExecutorKind::Shell(ref shell) => Box::new(ShellExecutor::new(shell, self.options)),
        };

        executor.calibrate()?;

        if self.options.interleave {
            return self.run_interleaved(&*executor);
        }

        for (number, cmd) in self.commands.iter().enumerate() {
            self.results.push(
                Benchmark::new(number, cmd, self.options, &*executor).run(self.results.first())?,
            );

            // We export results after each individual benchmark, because
            // we would risk losing them if a later benchmark fails.
            self.export_manager.write_results(&self.results, true)?;
        }

        Ok(())
    }

    fn run_interleaved(&mut self, executor: &dyn Executor) -> Result<()> {
        let commands = self.commands;
        let options = self.options;
        let mut runs: Vec<BenchmarkRun<'_>> = Vec::new();
        let progress_bar = (options.output_style != OutputStyleOption::Disabled).then(|| {
            let bar = get_progress_bar(
                commands.iter().count() as u64,
                "Preparing interleaved benchmarks",
                options.output_style,
            );
            set_benchmark_header(&bar, "Interleaved benchmarks".into());
            bar
        });

        let measured = (|| -> Result<()> {
            for (number, command) in commands.iter().enumerate() {
                runs.push(Benchmark::new(number, command, options, executor).start()?);
                if runs[number].is_complete() {
                    runs[number].cleanup()?;
                    self.export_interleaved_results(&runs)?;
                }
                if let Some(bar) = &progress_bar {
                    bar.inc(1);
                }
            }

            if let Some(bar) = &progress_bar {
                bar.set_length(
                    runs.iter()
                        .fold(0u64, |n, run| n.saturating_add(run.total_runs())),
                );
            }

            while runs.iter().any(|run| !run.is_complete()) {
                for number in 0..runs.len() {
                    let run = &mut runs[number];
                    if run.is_complete() {
                        continue;
                    }
                    if let Some(bar) = &progress_bar {
                        bar.set_message(format!("Running benchmark {}", number + 1));
                    }
                    run.step()?;
                    if let Some(bar) = &progress_bar {
                        bar.inc(1);
                    }
                    if run.is_complete() {
                        run.cleanup()?;
                        self.export_interleaved_results(&runs)?;
                    }
                }
            }
            Ok(())
        })();

        if let Some(bar) = &progress_bar {
            bar.finish_and_clear();
        }

        if let Err(error) = measured {
            // Keep all successful measurements, even when another command fails
            // before any individual benchmark has finished.
            if !runs.is_empty() {
                self.export_interleaved_results(&runs)?;
            }
            return Err(error);
        }

        for run in runs {
            self.results.push(run.finish(self.results.first())?);
        }
        self.export_manager.write_results(&self.results, true)?;

        Ok(())
    }

    fn export_interleaved_results(&self, runs: &[BenchmarkRun<'_>]) -> Result<()> {
        let partial: Vec<_> = runs.iter().map(BenchmarkRun::result).collect();
        self.export_manager.write_results(&partial, true)
    }

    pub fn final_export(&self) -> Result<()> {
        self.export_manager.write_results(&self.results, false)
    }
}

#[cfg(test)]
fn generate_results(args: &[&'static str]) -> Result<Vec<BenchmarkResult>> {
    use crate::cli::get_cli_arguments;

    let args = ["hyperfine", "--debug-mode", "--style=none"]
        .iter()
        .chain(args);
    let cli_arguments = get_cli_arguments(args);
    let mut options = Options::from_cli_arguments(&cli_arguments)?;

    assert_eq!(options.executor_kind, ExecutorKind::Mock(None));

    let commands = Commands::from_cli_arguments(&cli_arguments)?;
    let export_manager = ExportManager::from_cli_arguments(&cli_arguments, &options.metrics)?;

    options.validate_against_command_list(&commands)?;

    let mut scheduler = Scheduler::new(&commands, &options, &export_manager);

    scheduler.run_benchmarks()?;
    Ok(scheduler.results)
}

#[test]
fn scheduler_basic() -> Result<()> {
    insta::assert_yaml_snapshot!(generate_results(&["--runs=2", "sleep 0.123", "sleep 0.456"])?, @"
    - command: sleep 0.123
      measurements:
        - time_wall_clock:
            value: 0.123
            unit: second
          time_cpu:
            value: 0
            unit: second
          time_user:
            value: 0
            unit: second
          time_system:
            value: 0
            unit: second
          memory_peak_resident:
            value: 0
            unit: byte
          exit_code: 0
        - time_wall_clock:
            value: 0.123
            unit: second
          time_cpu:
            value: 0
            unit: second
          time_user:
            value: 0
            unit: second
          time_system:
            value: 0
            unit: second
          memory_peak_resident:
            value: 0
            unit: byte
          exit_code: 0
    - command: sleep 0.456
      measurements:
        - time_wall_clock:
            value: 0.456
            unit: second
          time_cpu:
            value: 0
            unit: second
          time_user:
            value: 0
            unit: second
          time_system:
            value: 0
            unit: second
          memory_peak_resident:
            value: 0
            unit: byte
          exit_code: 0
        - time_wall_clock:
            value: 0.456
            unit: second
          time_cpu:
            value: 0
            unit: second
          time_user:
            value: 0
            unit: second
          time_system:
            value: 0
            unit: second
          memory_peak_resident:
            value: 0
            unit: byte
          exit_code: 0
    ");

    Ok(())
}

#[test]
fn interleaved_commands_keep_independent_adaptive_run_counts() -> Result<()> {
    let results = generate_results(&[
        "--interleave",
        "--min-runs=2",
        "--min-benchmarking-time=1",
        "sleep 0.1",
        "sleep 0.25",
    ])?;

    assert_eq!(results[0].measurements.len(), 10);
    assert_eq!(results[1].measurements.len(), 4);
    assert_eq!(results[0].command, "sleep 0.1");
    assert_eq!(results[1].command, "sleep 0.25");
    Ok(())
}
