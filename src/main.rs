#![cfg_attr(
    all(windows, feature = "windows_process_extensions_main_thread_handle"),
    feature(windows_process_extensions_main_thread_handle)
)]

use std::env;
use std::io::{self, Write};

use benchmark::scheduler::Scheduler;
use cli::get_cli_arguments;
use command::Commands;
use error::ConsoleOutputError;
use export::ExportManager;
use options::{ExecutorKind, Options};

use anyhow::{bail, Result};
use colored::*;

pub mod benchmark;
pub mod cli;
pub mod command;
pub mod error;
pub mod export;
pub mod metric;
pub mod options;
pub mod outlier_detection;
pub mod output;
pub mod parameter;
pub mod quantity;
pub mod shell_syntax;
pub mod timer;
pub mod util;

fn run() -> Result<()> {
    // Enabled ANSI colors on Windows 10
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    let cli_arguments = get_cli_arguments(env::args_os());
    let mut options = Options::from_cli_arguments(&cli_arguments)?;
    let commands = Commands::from_cli_arguments(&cli_arguments)?;
    options.validate_against_command_list(&commands)?;

    if matches!(options.executor_kind, ExecutorKind::Raw)
        && !cli_arguments.get_flag("no-shell")
        && cli_arguments.get_one::<String>("shell").is_none()
    {
        for command in commands.iter() {
            let command_line = command.get_command_line();
            if let Some(operator) = shell_syntax::first_unquoted_operator(&command_line) {
                bail!(
                    concat!(
                        "Command '{command_line}' contains unquoted shell syntax ('{operator}').\n",
                        "Explicitly choose how to execute it:\n",
                        "  -S / --shell=default  Interpret shell syntax.\n",
                        "  -N / --shell=none     Run directly, passing '{operator}' as a literal argument.",
                    ),
                    command_line = command_line,
                    operator = operator,
                );
            }
        }
    }

    let export_manager = ExportManager::from_cli_arguments(&cli_arguments, options.metrics[0])?;

    let mut scheduler = Scheduler::new(&commands, &options, &export_manager);
    scheduler.run_benchmarks()?;
    scheduler.final_export()?;

    Ok(())
}

fn caused_by_broken_console_pipe(error: &anyhow::Error) -> bool {
    error
        .downcast_ref::<ConsoleOutputError>()
        .is_some_and(|e| e.0.kind() == io::ErrorKind::BrokenPipe)
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            // Exit quietly when a reader stops consuming console output early.
            if caused_by_broken_console_pipe(&e) {
                std::process::exit(0);
            }
            // The write to stderr can itself fail if stderr is closed; the
            // error message cannot be shown in that case anyway.
            let _ = writeln!(io::stderr(), "{} {:#}", "Error:".red(), e);
            std::process::exit(1);
        }
    }
}
