use std::cmp;
use std::collections::BTreeMap;
use std::env;
use std::io;

use atty::Stream;
use clap::ArgMatches;
use colored::*;

mod hyperfine;

use crate::hyperfine::app::get_arg_matches;
use crate::hyperfine::benchmark::{mean_shell_spawning_time, run_benchmark};
use crate::hyperfine::error::OptionsError;
use crate::hyperfine::export::{ExportManager, ExportType};
use crate::hyperfine::internal::{tokenize, write_benchmark_comparison};
use crate::hyperfine::parameter_range::get_parameterized_commands;
use crate::hyperfine::types::{
    CmdFailureAction, Command, HyperfineOptions, OutputStyleOption, ParameterValue,
};
use crate::hyperfine::units::Unit;

/// Print error message to stderr and terminate
pub fn error(message: &str) -> ! {
    eprintln!("{} {}", "Error:".red(), message);
    std::process::exit(1);
}

/// Runs the benchmark for the given commands
fn run(
    commands: &[Command<'_>],
    options: &HyperfineOptions,
    export_manager: &ExportManager,
) -> io::Result<()> {
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

        // Export (intermediate) results
        let ans = export_manager.write_results(&timing_results, options.time_unit);
        if let Err(e) = ans {
            error(&format!(
                "The following error occurred while exporting: {}",
                e
            ));
        }
    }

    // Print relative speed comparison
    if options.output_style != OutputStyleOption::Disabled {
        write_benchmark_comparison(&timing_results);
    }

    Ok(())
}

fn main() {
    let matches = get_arg_matches(env::args_os());
    let options = build_hyperfine_options(&matches);
    let commands = build_commands(&matches);
    let export_manager = match build_export_manager(&matches) {
        Ok(export_manager) => export_manager,
        Err(ref e) => error(&e.to_string()),
    };

    let res = match options {
        Ok(ref opts) => run(&commands, &opts, &export_manager),
        Err(ref e) => error(&e.to_string()),
    };

    match res {
        Ok(_) => {}
        Err(e) => error(&e.to_string()),
    }
}

/// Build the HyperfineOptions that correspond to the given ArgMatches
fn build_hyperfine_options<'a>(
    matches: &ArgMatches<'a>,
) -> Result<HyperfineOptions, OptionsError<'a>> {
    let mut options = HyperfineOptions::default();
    let param_to_u64 = |param| {
        matches
            .value_of(param)
            .map(|n| {
                u64::from_str_radix(n, 10).map_err(|e| OptionsError::NumericParsingError(param, e))
            })
            .transpose()
    };

    options.warmup_count = param_to_u64("warmup")?.unwrap_or(options.warmup_count);

    let mut min_runs = param_to_u64("min-runs")?;
    let mut max_runs = param_to_u64("max-runs")?;

    if let Some(runs) = param_to_u64("runs")? {
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
        Some("none") => OutputStyleOption::Disabled,
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
        OutputStyleOption::Full | OutputStyleOption::Color => colored::control::set_override(true),
        OutputStyleOption::Disabled => {}
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
fn build_export_manager(matches: &ArgMatches<'_>) -> io::Result<ExportManager> {
    let mut export_manager = ExportManager::new();
    {
        let mut add_exporter = |flag, exporttype| -> io::Result<()> {
            if let Some(filename) = matches.value_of(flag) {
                export_manager.add_exporter(exporttype, filename)?;
            }
            Ok(())
        };
        add_exporter("export-asciidoc", ExportType::Asciidoc)?;
        add_exporter("export-json", ExportType::Json)?;
        add_exporter("export-csv", ExportType::Csv)?;
        add_exporter("export-markdown", ExportType::Markdown)?;
    }
    Ok(export_manager)
}

/// Build the commands to benchmark
fn build_commands<'a>(matches: &'a ArgMatches<'_>) -> Vec<Command<'a>> {
    let command_names = matches.values_of("command-name");
    let command_strings = matches.values_of("command").unwrap();

    if let Some(args) = matches.values_of("parameter-scan") {
        let step_size = matches.value_of("parameter-step-size");
        match get_parameterized_commands(command_names, command_strings, args, step_size) {
            Ok(commands) => commands,
            Err(e) => error(&e.to_string()),
        }
    } else if let Some(args) = matches.values_of("parameter-list") {
        let command_names = command_names.map_or(vec![], |names| names.collect::<Vec<&str>>());

        let args: Vec<_> = args.collect();
        let param_names_and_values: Vec<(&str, Vec<String>)> = args
            .chunks_exact(2)
            .map(|pair| {
                let name = pair[0];
                let list_str = pair[1];
                (name, tokenize(list_str))
            })
            .collect();
        {
            let dupes = find_dupes(param_names_and_values.iter().map(|(name, _)| *name));
            if !dupes.is_empty() {
                error(&format!("duplicate parameter names: {}", &dupes.join(", ")))
            }
        }
        let command_list = command_strings.collect::<Vec<&str>>();

        let dimensions: Vec<usize> = std::iter::once(command_list.len())
            .chain(
                param_names_and_values
                    .iter()
                    .map(|(_, values)| values.len()),
            )
            .collect();
        let param_space_size = dimensions.iter().product();
        if param_space_size == 0 {
            return Vec::new();
        }

        // `--command-name` should appear exactly once or same count with parameters.
        let command_name_count = command_names.len();
        if command_name_count > 1 && command_name_count != param_space_size {
            let err = OptionsError::DifferentCommandNameCountWithParameters(
                command_name_count,
                param_space_size,
            );
            error(&err.to_string());
        }

        let mut i = 0;
        let mut commands = Vec::with_capacity(param_space_size);
        let mut index = vec![0usize; dimensions.len()];
        'outer: loop {
            let name = command_names
                .get(i)
                .or_else(|| command_names.get(0))
                .map(|s| *s);
            i += 1;

            let (command_index, params_indices) = index.split_first().unwrap();
            let parameters = param_names_and_values
                .iter()
                .zip(params_indices)
                .map(|((name, values), i)| (*name, ParameterValue::Text(values[*i].clone())))
                .collect();
            commands.push(Command::new_parametrized(
                name,
                command_list[*command_index],
                parameters,
            ));

            // Increment index, exiting loop on overflow.
            for (i, n) in index.iter_mut().zip(dimensions.iter()) {
                *i += 1;
                if *i < *n {
                    continue 'outer;
                } else {
                    *i = 0;
                }
            }
            break 'outer;
        }

        commands
    } else {
        let command_names = command_names.map_or(vec![], |names| names.collect::<Vec<&str>>());
        if command_names.len() > command_strings.len() {
            let err = OptionsError::TooManyCommandNames(command_strings.len());
            error(&err.to_string());
        }

        let command_list = command_strings.collect::<Vec<&str>>();
        let mut commands = Vec::with_capacity(command_list.len());
        for (i, s) in command_list.iter().enumerate() {
            commands.push(Command::new(command_names.get(i).copied(), &s));
        }
        commands
    }
}

/// Finds all the strings that appear multiple times in the input iterator, returning them in
/// sorted order. If no string appears more than once, the result is an empty vector.
fn find_dupes<'a, I: IntoIterator<Item = &'a str>>(i: I) -> Vec<&'a str> {
    let mut counts = BTreeMap::<&'a str, usize>::new();
    for s in i {
        *counts.entry(s).or_default() += 1;
    }
    counts
        .into_iter()
        .filter_map(|(k, n)| if n > 1 { Some(k) } else { None })
        .collect()
}

#[test]
fn test_build_commands_cross_product() {
    let matches = get_arg_matches(vec![
        "hyperfine",
        "-L",
        "foo",
        "a,b",
        "-L",
        "bar",
        "z,y",
        "echo {foo} {bar}",
        "printf '%s\n' {foo} {bar}",
    ]);
    let result = build_commands(&matches);

    // Iteration order: command list first, then parameters in listed order (here, "foo" before
    // "bar", which is distinct from their sorted order), with parameter values in listed order.
    let pv = |s: &str| ParameterValue::Text(s.to_string());
    let cmd = |cmd: usize, foo: &str, bar: &str| {
        let expression = ["echo {foo} {bar}", "printf '%s\n' {foo} {bar}"][cmd];
        let params = vec![("foo", pv(foo)), ("bar", pv(bar))];
        Command::new_parametrized(None, expression, params)
    };
    let expected = vec![
        cmd(0, "a", "z"),
        cmd(1, "a", "z"),
        cmd(0, "b", "z"),
        cmd(1, "b", "z"),
        cmd(0, "a", "y"),
        cmd(1, "a", "y"),
        cmd(0, "b", "y"),
        cmd(1, "b", "y"),
    ];
    assert_eq!(result, expected);
}

#[test]
fn test_build_parameter_list_commands() {
    let matches = get_arg_matches(vec![
        "hyperfine",
        "echo {foo}",
        "--parameter-list",
        "foo",
        "1,2",
        "--command-name",
        "name-{foo}",
    ]);
    let commands = build_commands(&matches);
    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].get_name(), "name-1");
    assert_eq!(commands[1].get_name(), "name-2");
    assert_eq!(commands[0].get_shell_command(), "echo 1");
    assert_eq!(commands[1].get_shell_command(), "echo 2");
}

#[test]
fn test_build_parameter_range_commands() {
    let matches = get_arg_matches(vec![
        "hyperfine",
        "echo {val}",
        "--parameter-scan",
        "val",
        "1",
        "2",
        "--parameter-step-size",
        "1",
        "--command-name",
        "name-{val}",
    ]);
    let commands = build_commands(&matches);
    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].get_name(), "name-1");
    assert_eq!(commands[1].get_name(), "name-2");
    assert_eq!(commands[0].get_shell_command(), "echo 1");
    assert_eq!(commands[1].get_shell_command(), "echo 2");
}
