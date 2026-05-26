use super::benchmark_result::{merge_parameter_benchmark_results, BenchmarkResult};
use super::executor::{Executor, MockExecutor, RawExecutor, ShellExecutor};
use super::{relative_speed, Benchmark};
use colored::*;
use std::cmp::Ordering;

use crate::command::{Command, Commands};
use crate::export::ExportManager;
use crate::options::{ExecutorKind, Options, OutputStyleOption, SortOrder};

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

        if self.options.aggregate_parameter_runs {
            let template = self
                .commands
                .template_expression()
                .expect("validated in Options::validate_against_command_list");

            if let Some(ref_cmd) = reference.as_ref() {
                self.results
                    .push(Benchmark::new(0, ref_cmd, self.options, &*executor).run()?);
            }

            let start_index = if reference.is_some() { 1 } else { 0 };
            let mut parameter_results = Vec::with_capacity(self.commands.num_commands(false));

            for (offset, cmd) in self.commands.iter().enumerate() {
                parameter_results.push(
                    Benchmark::new(start_index + offset, cmd, self.options, &*executor).run()?,
                );
            }

            self.results.push(merge_parameter_benchmark_results(
                parameter_results,
                template,
            ));
            self.export_manager.write_results(&self.results, true)?;

            return Ok(());
        }

        for (number, cmd) in reference.iter().chain(self.commands.iter()).enumerate() {
            self.results
                .push(Benchmark::new(number, cmd, self.options, &*executor).run()?);

            // We export results after each individual benchmark, because
            // we would risk losing them if a later benchmark fails.
            self.export_manager.write_results(&self.results, true)?;
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

        let reference = self
            .options
            .reference_command
            .as_ref()
            .map(|_| &self.results[0])
            .unwrap_or_else(|| relative_speed::fastest_of(&self.results));

        if let Some(annotated_results) = relative_speed::compute_with_check_from_reference(
            &self.results,
            reference,
            self.options.sort_order_speed_comparison,
        ) {
            match self.options.sort_order_speed_comparison {
                SortOrder::MeanTime => {
                    println!("{}", "Summary".bold());

                    let reference = annotated_results.iter().find(|r| r.is_reference).unwrap();
                    let others = annotated_results.iter().filter(|r| !r.is_reference);

                    println!(
                        "  {} ran",
                        reference.result.command_with_unused_parameters.cyan()
                    );

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
                        println!(
                            "{} {}",
                            comparator,
                            &item.result.command_with_unused_parameters.magenta()
                        );
                    }
                }
                SortOrder::Command => {
                    println!("{}", "Relative speed comparison".bold());

                    for item in annotated_results {
                        println!(
                            "  {}{}  {}",
                            format!("{:10.2}", item.relative_speed).bold().green(),
                            if item.is_reference {
                                "        ".into()
                            } else if let Some(stddev) = item.relative_speed_stddev {
                                format!(" ± {}", format!("{stddev:5.2}").green())
                            } else {
                                "        ".into()
                            },
                            &item.result.command_with_unused_parameters,
                        );
                    }
                }
            }
        } else {
            eprintln!(
                "{}: The benchmark comparison could not be computed as some benchmark times are zero. \
                 This could be caused by background interference during the initial calibration phase \
                 of hyperfine, in combination with very fast commands (faster than a few milliseconds). \
                 Try to re-run the benchmark on a quiet system. If you did not do so already, try the \
                 --shell=none/-N option. If it does not help either, you command is most likely too fast \
                 to be accurately benchmarked by hyperfine.",
                 "Note".bold().red()
            );
        }
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
      mean: 0.123
      stddev: 0
      median: 0.123
      user: 0
      system: 0
      min: 0.123
      max: 0.123
      times:
        - 0.123
        - 0.123
      memory_usage_byte:
        - 0
        - 0
      exit_codes:
        - 0
        - 0
    - command: sleep 0.456
      mean: 0.456
      stddev: 0
      median: 0.456
      user: 0
      system: 0
      min: 0.456
      max: 0.456
      times:
        - 0.456
        - 0.456
      memory_usage_byte:
        - 0
        - 0
      exit_codes:
        - 0
        - 0
    "#);

    Ok(())
}

#[test]
fn scheduler_aggregate_parameter_runs() -> Result<()> {
    insta::assert_yaml_snapshot!(
        generate_results(&[
            "--runs=1",
            "--aggregate-parameter-runs",
            "--parameter-scan",
            "index",
            "1",
            "3",
            "sleep {index}.123",
        ])?,
        @r#"
    - command: "sleep {index}.123"
      mean: 2.123
      stddev: 1
      median: 2.123
      user: 0
      system: 0
      min: 1.123
      max: 3.123
      times:
        - 1.123
        - 2.123
        - 3.123
      memory_usage_byte:
        - 0
        - 0
        - 0
      exit_codes:
        - 0
        - 0
        - 0
    "#
    );

    Ok(())
}
