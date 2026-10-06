use super::benchmark_result::BenchmarkResult;
use super::executor::{Executor, MockExecutor, RawExecutor, ShellExecutor};
use super::Benchmark;

use crate::command::Commands;
use crate::export::ExportManager;
use crate::options::{ExecutorKind, Options};

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
    let export_manager = ExportManager::from_cli_arguments(&cli_arguments, options.metrics[0])?;

    options.validate_against_command_list(&commands)?;

    let mut scheduler = Scheduler::new(&commands, &options, &export_manager);

    scheduler.run_benchmarks()?;
    Ok(scheduler.results)
}

#[test]
fn scheduler_basic() -> Result<()> {
    insta::assert_yaml_snapshot!(generate_results(&["--runs=2", "sleep 0.123", "sleep 0.456"])?, @r#"
    - command: sleep 0.123
      measurements:
        - time_wall_clock:
            value: 0.123
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
    "#);

    Ok(())
}
