use colored::*;

use super::benchmark_result::BenchmarkResult;
use super::executor::{Executor, MockExecutor, RawExecutor, ShellExecutor};
use super::{relative_speed, Benchmark};

use crate::command::Commands;
use crate::export::ExportManager;
use crate::options::{ExecutorKind, Options, OutputStyleOption};

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
            self.results
                .push(Benchmark::new(number, cmd, self.options, &*executor).run()?);

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

        if let Some(annotated_results) = relative_speed::compute(&self.results) {
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
}
