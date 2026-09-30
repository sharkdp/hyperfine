#![cfg_attr(
    all(windows, feature = "windows_process_extensions_main_thread_handle"),
    feature(windows_process_extensions_main_thread_handle)
)]

use std::env;
use std::io::{self, Write};

use benchmark::scheduler::Scheduler;
use cli::get_cli_arguments;
use command::Commands;
use export::ExportManager;
use options::Options;

use anyhow::Result;
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

/// Check whether `error` or one of its causes is an `io::Error` of kind
/// `BrokenPipe`. This happens when the process on the reading end of a
/// pipe (e.g. `hyperfine … | head -n 1`) terminates before hyperfine is
/// done writing its results.
fn caused_by_broken_pipe(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        cause
            .downcast_ref::<io::Error>()
            .is_some_and(|e| e.kind() == io::ErrorKind::BrokenPipe)
    })
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            // When writing to a closed pipe, exit quietly with a zero exit
            // code — the same behavior that non-Rust CLI tools exhibit when
            // they are terminated by `SIGPIPE`.
            if caused_by_broken_pipe(&e) {
                std::process::exit(0);
            }
            // The write to stderr can itself fail if stderr is closed; the
            // error message cannot be shown in that case anyway.
            let _ = writeln!(io::stderr(), "{} {:#}", "Error:".red(), e);
            std::process::exit(1);
        }
    }
}
