extern crate atty;

#[macro_use]
extern crate cfg_if;

#[macro_use]
extern crate clap;
extern crate colored;
extern crate csv;
extern crate indicatif;
extern crate serde;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate statistical;

// Os-specific dependencies
cfg_if! {
    if #[cfg(windows)] {
        extern crate winapi;
    } else {
        extern crate libc;
    }
}

#[cfg(test)]
#[macro_use]
extern crate approx;

use std::cmp;
use std::env;
use std::error::Error;
use std::io;
use std::ops::Range;

use atty::Stream;
use clap::ArgMatches;
use colored::*;

mod hyperfine;

use hyperfine::app::get_arg_matches;
use hyperfine::benchmark::{mean_shell_spawning_time, run_benchmark};
use hyperfine::error::{OptionsError, ParameterScanError};
use hyperfine::export::{ExportManager, ExportType};
use hyperfine::internal::write_benchmark_comparison;
use hyperfine::types::{
    BenchmarkResult, CmdFailureAction, Command, HyperfineOptions, OutputStyleOption,
};

/// Print error message to stderr and terminate
pub fn error(message: &str) -> ! {
    eprintln!("{} {}", "Error:".red(), message);
    std::process::exit(1);
}

/// Runs the benchmark for the given commands
fn run(commands: &Vec<Command>, options: &HyperfineOptions) -> io::Result<Vec<BenchmarkResult>> {
    let shell_spawning_time = mean_shell_spawning_time(&options.shell, &options.output_style, options.show_output)?;

    let mut timing_results = vec![];

    // Run the benchmarks
    for (num, cmd) in commands.iter().enumerate() {
        timing_results.push(run_benchmark(num, cmd, shell_spawning_time, options)?);
    }

    Ok(timing_results)
}

/// A function to read the `--parameter-scan` arguments
fn parse_parameter_scan_args<'a>(
    mut vals: clap::Values<'a>,
) -> Result<(&'a str, Range<i32>), ParameterScanError> {
    let param_name = vals.next().unwrap();
    let param_min: i32 = vals.next().unwrap().parse()?;
    let param_max: i32 = vals.next().unwrap().parse()?;

    const MAX_PARAMETERS: i32 = 100000;
    if param_max - param_min > MAX_PARAMETERS {
        return Err(ParameterScanError::TooLarge);
    }

    if param_max < param_min {
        return Err(ParameterScanError::EmptyRange);
    }

    return Ok((param_name, param_min..(param_max + 1)));
}

fn main() {
    let matches = get_arg_matches(env::args_os());
    let options = build_hyperfine_options(&matches);
    let export_manager = build_export_manager(&matches);
    let commands = build_commands(&matches);

    let res = match options {
        Ok(opts) => run(&commands, &opts),
        Err(e) => error(e.description()),
    };

    match res {
        Ok(timing_results) => {
            write_benchmark_comparison(&timing_results);
            let ans = export_manager.write_results(timing_results);
            if let Err(e) = ans {
                error(&format!(
                    "The following error occurred while exporting: {}",
                    e.description()
                ));
            }
        }
        Err(e) => error(e.description()),
    }
}

/// Build the HyperfineOptions that correspond to the given ArgMatches
fn build_hyperfine_options(matches: &ArgMatches) -> Result<HyperfineOptions, OptionsError> {
    let mut options = HyperfineOptions::default();
    let str_to_u64 = |n| u64::from_str_radix(n, 10).ok();

    options.warmup_count = matches
        .value_of("warmup")
        .and_then(&str_to_u64)
        .unwrap_or(options.warmup_count);

    let mut min_runs = matches.value_of("min-runs").and_then(&str_to_u64);
    let mut max_runs = matches.value_of("max-runs").and_then(&str_to_u64);

    if let Some(runs) = matches.value_of("runs").and_then(&str_to_u64) {
        min_runs = Some(runs);
        max_runs = Some(runs);
    }

    match (min_runs, max_runs) {
        (Some(min), _) if min < 2 => {
            // We need at least two runs to compute a variance.
            return Err(OptionsError::RunsBelowTwo);
        }
        (Some(min), None) => {
            options.runs.min = min;
        }
        (_, Some(max)) if max < 2 => {
            // We need at least two runs to compute a variance.
            return Err(OptionsError::RunsBelowTwo);
        }
        (None, Some(max)) => {
            // Since the minimum was not explicit we lower it if max is below the default min.
            options.runs.min = cmp::min(options.runs.min, max);
            options.runs.max = Some(max);
        }
        (Some(min), Some(max)) if min > max => {
            return Err(OptionsError::EmptyRunsRange);
        }
        (Some(min), Some(max)) => {
            options.runs.min = min;
            options.runs.max = Some(max);
        }
        (None, None) => {}
    };

    options.preparation_command = matches.value_of("prepare").map(String::from);

    options.show_output = matches.is_present("show-output");

    options.output_style = match matches.value_of("style") {
        Some("full") => OutputStyleOption::Full,
        Some("basic") => OutputStyleOption::Basic,
        Some("nocolor") => OutputStyleOption::NoColor,
        Some("color") => OutputStyleOption::Color,
        _ => {
            if !options.show_output && atty::is(Stream::Stdout) {
                OutputStyleOption::Full
            } else {
                OutputStyleOption::Basic
            }
        }
    };

    // We default Windows to NoColor if full had been specified.
    if cfg!(windows) && options.output_style == OutputStyleOption::Full {
        options.output_style = OutputStyleOption::NoColor;
    }

    match options.output_style {
        OutputStyleOption::Basic | OutputStyleOption::NoColor => {
            colored::control::set_override(false)
        }
        _ => {}
    };

    options.shell = matches.value_of("shell").unwrap_or(&options.shell).to_string();

    if matches.is_present("ignore-failure") {
        options.failure_action = CmdFailureAction::Ignore;
    }

    Ok(options)
}

/// Build the ExportManager that will export the results specified
/// in the given ArgMatches
fn build_export_manager(matches: &ArgMatches) -> ExportManager {
    let mut export_manager = ExportManager::new();
    if let Some(filename) = matches.value_of("export-json") {
        export_manager.add_exporter(ExportType::Json, filename);
    }
    if let Some(filename) = matches.value_of("export-csv") {
        export_manager.add_exporter(ExportType::Csv, filename);
    }
    if let Some(filename) = matches.value_of("export-markdown") {
        export_manager.add_exporter(ExportType::Markdown, filename);
    }
    export_manager
}

/// Build the commands to benchmark
fn build_commands<'a>(matches: &'a ArgMatches) -> Vec<Command<'a>> {
    let command_strings = matches.values_of("command").unwrap();

    let commands = if let Some(args) = matches.values_of("parameter-scan") {
        match parse_parameter_scan_args(args) {
            Ok((param_name, param_range)) => {
                let mut commands = vec![];
                let command_strings = command_strings.collect::<Vec<&str>>();
                for value in param_range.start..param_range.end {
                    for ref cmd in &command_strings {
                        commands.push(Command::new_parametrized(cmd, param_name, value));
                    }
                }
                commands
            }
            Err(e) => error(e.description()),
        }
    } else {
        command_strings.map(|c| Command::new(c)).collect()
    };
    commands
}
