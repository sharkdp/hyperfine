#![cfg_attr(
    all(windows, feature = "windows_process_extensions_main_thread_handle"),
    feature(windows_process_extensions_main_thread_handle)
)]

use std::env;
use std::io::{self, Write};

use benchmark::scheduler::Scheduler;
use cli::{build_command, get_cli_arguments};
use command::Commands;
use error::ConsoleOutputError;
use export::ExportManager;
use options::Options;

use anyhow::Result;
use clap_complete::{generate, shells::Shell};
use colored::*;

pub mod benchmark;
pub mod cli;
pub mod command;
pub mod error;
pub mod export;
pub mod options;
pub mod outlier_detection;
pub mod output;
pub mod parameter;
pub mod timer;
pub mod util;

fn run() -> Result<()> {
    // Enabled ANSI colors on Windows 10
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    let cli_arguments = get_cli_arguments(env::args_os());
    if let Some(("generate-shell-completion", subcommand)) = cli_arguments.subcommand() {
        let shell = subcommand
            .get_one::<Shell>("shell")
            .expect("required by clap");
        let mut command = build_command();
        generate(*shell, &mut command, "hyperfine", &mut std::io::stdout());
        return Ok(());
    }

    let mut options = Options::from_cli_arguments(&cli_arguments)?;
    let commands = Commands::from_cli_arguments(&cli_arguments)?;
    let export_manager = ExportManager::from_cli_arguments(
        &cli_arguments,
        options.time_unit,
        options.sort_order_exports,
    )?;

    options.validate_against_command_list(&commands)?;

    let mut scheduler = Scheduler::new(&commands, &options, &export_manager);
    scheduler.run_benchmarks()?;
    scheduler.print_relative_speed_comparison()?;
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
