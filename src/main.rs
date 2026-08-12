#![cfg_attr(
    all(windows, feature = "windows_process_extensions_main_thread_handle"),
    feature(windows_process_extensions_main_thread_handle)
)]

use std::env;

use benchmark::scheduler::Scheduler;
use clap::ArgMatches;
use clap_complete::Shell;
use cli::{build_command, get_cli_arguments};
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

    if let Some(shell) = cli_arguments.get_one::<String>("generate-completions") {
        let shell = parse_shell(shell)?;
        let mut command = build_command();
        clap_complete::generate(shell, &mut command, "hyperfine", &mut std::io::stdout());
        return Ok(());
    }

    run_benchmarks(&cli_arguments)
}

fn run_benchmarks(cli_arguments: &ArgMatches) -> Result<()> {
    let mut options = Options::from_cli_arguments(cli_arguments)?;
    let commands = Commands::from_cli_arguments(cli_arguments)?;
    let export_manager = ExportManager::from_cli_arguments(
        cli_arguments,
        options.time_unit,
        options.sort_order_exports,
    )?;

    options.validate_against_command_list(&commands)?;

    let mut scheduler = Scheduler::new(&commands, &options, &export_manager);
    scheduler.run_benchmarks()?;
    scheduler.print_relative_speed_comparison();
    scheduler.final_export()?;

    Ok(())
}

fn parse_shell(name: &str) -> Result<Shell> {
    match name {
        "bash" => Ok(Shell::Bash),
        "zsh" => Ok(Shell::Zsh),
        "fish" => Ok(Shell::Fish),
        "powershell" => Ok(Shell::PowerShell),
        "elvish" => Ok(Shell::Elvish),
        other => Err(anyhow::anyhow!("unsupported shell: {other}")),
    }
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{} {:#}", "Error:".red(), e);
            std::process::exit(1);
        }
    }
}
