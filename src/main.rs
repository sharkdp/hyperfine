use std::env;

use app::get_arg_matches;
use benchmark::schedule::run_benchmarks_and_print_comparison;
use command::build_commands;
use export::ExportManager;
use options::Options;

use anyhow::Result;
use colored::*;

pub mod app;
pub mod benchmark;
pub mod command;
pub mod error;
pub mod export;
pub mod options;
pub mod outlier_detection;
pub mod output;
pub mod parameter;
pub mod shell;
pub mod timer;
pub mod util;

fn run() -> Result<()> {
    // Enabled ANSI colors on Windows 10
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    let matches = get_arg_matches(env::args_os());
    let options = Options::from_cli_arguments(&matches)?;
    let commands = build_commands(&matches)?;
    let export_manager = ExportManager::from_cli_arguments(&matches)?;

    run_benchmarks_and_print_comparison(&commands, &options, &export_manager)
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
