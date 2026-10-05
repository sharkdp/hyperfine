use super::benchmark_result::BenchmarkResult;
use super::executor::{Executor, MockExecutor, RawExecutor, ShellExecutor};
use super::{relative_speed, Benchmark};
use colored::*;
use std::cmp::Ordering;
use std::io::{self, Write};

use crate::command::{Command, Commands};
use crate::export::ExportManager;
use crate::options::{ExecutorKind, Options, OutputStyleOption, SortOrder};
use crate::output::console_writeln;

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

        let reference = self
            .options
            .reference_command
            .as_ref()
            .map(|cmd| Command::new(self.options.reference_name.as_deref(), cmd));

        executor.calibrate()?;

        for (number, cmd) in reference.iter().chain(self.commands.iter()).enumerate() {
            self.results
                .push(Benchmark::new(number, cmd, self.options, &*executor).run()?);

            // We export results after each individual benchmark, because
            // we would risk losing them if a later benchmark fails.
            self.export_manager
                .write_results(&self.results, true, self.options.reference_index)?;
        }

        Ok(())
    }

    pub fn print_relative_speed_comparison(&self) -> Result<()> {
        if self.options.output_style == OutputStyleOption::Disabled {
            return Ok(());
        }

        if self.results.len() < 2 {
            return Ok(());
        }

        let reference = self
            .options
            .reference_index
            .map(|index| &self.results[index])
            .unwrap_or_else(|| relative_speed::fastest_of(&self.results));

        if let Some(annotated_results) = relative_speed::compute_with_check_from_reference(
            &self.results,
            reference,
            self.options.sort_order_speed_comparison,
        ) {
            let mut stdout = io::stdout().lock();
            match self.options.sort_order_speed_comparison {
                SortOrder::MeanTime => {
                    console_writeln!(stdout, "{}", "Summary".bold())?;

                    let reference = annotated_results.iter().find(|r| r.is_reference).unwrap();
                    let others = annotated_results.iter().filter(|r| !r.is_reference);

                    console_writeln!(stdout, "  {} ran", reference.result.display_name.cyan())?;

                    for item in others {
                        let stddev = if let Some(stddev) = item.relative_speed_stddev {
                            format!(" ± {}", format!("{:.2}", stddev).green())
                        } else {
                            "".into()
                        };
                        let comparator = match item.relative_ordering {
                            Ordering::Less => format!(
                                "{}{} times slower than",
                                format!("{:8.2}", item.relative_speed).bold().green(),
                                stddev
                            ),
                            Ordering::Greater => format!(
                                "{}{} times faster than",
                                format!("{:8.2}", item.relative_speed).bold().green(),
                                stddev
                            ),
                            Ordering::Equal => format!(
                                "    As fast ({}{}) as",
                                format!("{:.2}", item.relative_speed).bold().green(),
                                stddev
                            ),
                        };
                        console_writeln!(
                            stdout,
                            "{} {}",
                            comparator,
                            &item.result.display_name.magenta()
                        )?;
                    }
                }
                SortOrder::Command => {
                    console_writeln!(stdout, "{}", "Relative speed comparison".bold())?;

                    for item in annotated_results {
                        let relationship = if self.options.reference_index.is_some() {
                            item.reference_label()
                        } else {
                            ""
                        };
                        console_writeln!(
                            stdout,
                            "  {}{}  {}{}",
                            format!("{:10.2}", item.relative_speed).bold().green(),
                            if item.is_reference {
                                "        ".into()
                            } else if let Some(stddev) = item.relative_speed_stddev {
                                format!(" ± {}", format!("{stddev:5.2}").green())
                            } else {
                                "        ".into()
                            },
                            item.result.display_name,
                            relationship,
                        )?;
                    }
                }
            }
        } else {
            console_writeln!(
                io::stderr(),
                "{}: The benchmark comparison could not be computed as some benchmark times are zero. \
                 This could be caused by background interference during the initial calibration phase \
                 of hyperfine, in combination with very fast commands (faster than a few milliseconds). \
                 Try to re-run the benchmark on a quiet system. If you did not do so already, try the \
                 --shell=none/-N option. If it does not help either, you command is most likely too fast \
                 to be accurately benchmarked by hyperfine.",
                 "Note".bold().red()
            )?;
        }

        Ok(())
    }

    pub fn final_export(&self) -> Result<()> {
        self.export_manager
            .write_results(&self.results, false, self.options.reference_index)
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
    let export_manager = ExportManager::from_cli_arguments(
        &cli_arguments,
        options.time_unit,
        options.sort_order_exports,
    )?;

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
