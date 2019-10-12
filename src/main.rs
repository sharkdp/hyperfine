use std::cmp;
use std::env;
use std::error::Error;
use std::io;

use atty::Stream;
use clap::ArgMatches;
use colored::*;

mod hyperfine;

use crate::hyperfine::app::get_arg_matches;
use crate::hyperfine::benchmark::{mean_shell_spawning_time, run_benchmark};
use crate::hyperfine::error::OptionsError;
use crate::hyperfine::export::{ExportManager, ExportType};
use crate::hyperfine::internal::write_benchmark_comparison;
use crate::hyperfine::parameter_range::get_parameterized_commands;
use crate::hyperfine::types::{
    BenchmarkResult, CmdFailureAction, Command, HyperfineOptions, OutputStyleOption,
};
use crate::hyperfine::units::Unit;

/// Print error message to stderr and terminate
pub fn error(message: &str) -> ! {
    eprintln!("{} {}", "Error:".red(), message);
    std::process::exit(1);
}

/// Runs the benchmark for the given commands
fn run(commands: &[Command<'_>], options: &HyperfineOptions) -> io::Result<Vec<BenchmarkResult>> {
    let shell_spawning_time =
        mean_shell_spawning_time(&options.shell, options.output_style, options.show_output)?;

    let mut timing_results = vec![];

    if let Some(preparation_command) = &options.preparation_command {
        if preparation_command.len() > 1 && commands.len() != preparation_command.len() {
            error(
                "The '--prepare' option has to be provided just once or N times, where N is the \
                 number of benchmark commands.",
            );
        }
    }

    // Run the benchmarks
    for (num, cmd) in commands.iter().enumerate() {
        timing_results.push(run_benchmark(num, cmd, shell_spawning_time, options)?);
    }

    Ok(timing_results)
}

fn main() {
    let matches = get_arg_matches(env::args_os());
    let options = build_hyperfine_options(&matches);
    let export_manager = build_export_manager(&matches);
    let commands = build_commands(&matches);

    let res = match options {
        Ok(ref opts) => run(&commands, &opts),
        Err(ref e) => error(e.description()),
    };

    match res {
        Ok(timing_results) => {
            let unwrapped = options.unwrap();
            if unwrapped.output_style != OutputStyleOption::None { write_benchmark_comparison(&timing_results); }
            let ans = export_manager.write_results(timing_results, unwrapped.time_unit);
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
fn build_hyperfine_options(matches: &ArgMatches<'_>) -> Result<HyperfineOptions, OptionsError> {
    let mut options = HyperfineOptions::default();
    let param_to_u64 = |param| {
        matches
            .value_of(param)
            .and_then(|n| u64::from_str_radix(n, 10).ok())
    };

    options.warmup_count = param_to_u64("warmup").unwrap_or(options.warmup_count);

    let mut min_runs = param_to_u64("min-runs");
    let mut max_runs = param_to_u64("max-runs");

    if let Some(runs) = param_to_u64("runs") {
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

    options.preparation_command = matches
        .values_of("prepare")
        .map(|values| values.map(String::from).collect::<Vec<String>>());

    options.cleanup_command = matches.value_of("cleanup").map(String::from);

    options.show_output = matches.is_present("show-output");

    options.output_style = match matches.value_of("style") {
        Some("full") => OutputStyleOption::Full,
        Some("basic") => OutputStyleOption::Basic,
        Some("nocolor") => OutputStyleOption::NoColor,
        Some("color") => OutputStyleOption::Color,
        Some("none") => OutputStyleOption::None,
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

    options.shell = matches
        .value_of("shell")
        .unwrap_or(&options.shell)
        .to_string();

    if matches.is_present("ignore-failure") {
        options.failure_action = CmdFailureAction::Ignore;
    }

    options.time_unit = match matches.value_of("time-unit") {
        Some("millisecond") => Some(Unit::MilliSecond),
        Some("second") => Some(Unit::Second),
        _ => None,
    };

    Ok(options)
}

/// Build the ExportManager that will export the results specified
/// in the given ArgMatches
fn build_export_manager(matches: &ArgMatches<'_>) -> ExportManager {
    let mut export_manager = ExportManager::new();
    {
        let mut add_exporter = |flag, exporttype| {
            if let Some(filename) = matches.value_of(flag) {
                export_manager.add_exporter(exporttype, filename);
            }
        };
        add_exporter("export-asciidoc", ExportType::Asciidoc);
        add_exporter("export-json", ExportType::Json);
        add_exporter("export-csv", ExportType::Csv);
        add_exporter("export-markdown", ExportType::Markdown);
    }
    export_manager
}

/// Build the commands to benchmark
fn build_commands<'a>(matches: &'a ArgMatches<'_>) -> Vec<Command<'a>> {
    let command_strings = matches.values_of("command").unwrap();

    if let Some(args) = matches.values_of("parameter-scan") {
        let step_size = matches.value_of("parameter-step-size");
        match get_parameterized_commands(command_strings, args, step_size) {
            Ok(commands) => commands,
            Err(e) => error(e.description()),
        }
    } else {
        command_strings.map(Command::new).collect()
    }
}
