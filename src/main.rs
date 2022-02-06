use std::collections::BTreeMap;
use std::env;

use clap::ArgMatches;
use colored::*;

pub mod app;
pub mod benchmark;
pub mod benchmark_result;
pub mod command;
pub mod error;
pub mod export;
pub mod format;
pub mod min_max;
pub mod options;
pub mod outlier_detection;
pub mod parameter_range;
pub mod progress_bar;
pub mod relative_speed;
pub mod shell;
pub mod timer;
pub mod tokenize;
pub mod types;
pub mod units;
pub mod warnings;

use app::get_arg_matches;
use benchmark::{mean_shell_spawning_time, run_benchmark};
use benchmark_result::BenchmarkResult;
use command::Command;
use error::OptionsError;
use export::ExportManager;
use options::{Options, OutputStyleOption};
use parameter_range::get_parameterized_commands;
use tokenize::tokenize;
use types::ParameterValue;

use anyhow::{bail, Result};

pub fn write_benchmark_comparison(results: &[BenchmarkResult]) {
    if results.len() < 2 {
        return;
    }

    if let Some(mut annotated_results) = relative_speed::compute(results) {
        annotated_results.sort_by(|l, r| relative_speed::compare_mean_time(l.result, r.result));

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

fn run_benchmarks_and_print_comparison(
    commands: &[Command<'_>],
    options: &Options,
    export_manager: &ExportManager,
) -> Result<()> {
    let shell_spawning_time =
        mean_shell_spawning_time(&options.shell, options.output_style, options.show_output)?;

    let mut timing_results = vec![];

    if let Some(preparation_command) = &options.preparation_command {
        if preparation_command.len() > 1 && commands.len() != preparation_command.len() {
            bail!(
                "The '--prepare' option has to be provided just once or N times, where N is the \
                 number of benchmark commands."
            );
        }
    }

    // Run the benchmarks
    for (num, cmd) in commands.iter().enumerate() {
        timing_results.push(run_benchmark(num, cmd, shell_spawning_time, options)?);

        // Export (intermediate) results
        export_manager.write_results(&timing_results, options.time_unit)?;
    }

    // Print relative speed comparison
    if options.output_style != OutputStyleOption::Disabled {
        write_benchmark_comparison(&timing_results);
    }

    Ok(())
}

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

/// Build the commands to benchmark
fn build_commands(matches: &ArgMatches) -> Result<Vec<Command>> {
    let command_names = matches.values_of("command-name");
    let command_strings = matches.values_of("command").unwrap();

    if let Some(args) = matches.values_of("parameter-scan") {
        let step_size = matches.value_of("parameter-step-size");
        Ok(get_parameterized_commands(
            command_names,
            command_strings,
            args,
            step_size,
        )?)
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
                bail!("Duplicate parameter names: {}", &dupes.join(", "));
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
            return Ok(Vec::new());
        }

        // `--command-name` should appear exactly once or exactly B times,
        // where B is the total number of benchmarks.
        let command_name_count = command_names.len();
        if command_name_count > 1 && command_name_count != param_space_size {
            return Err(OptionsError::UnexpectedCommandNameCount(
                command_name_count,
                param_space_size,
            )
            .into());
        }

        let mut i = 0;
        let mut commands = Vec::with_capacity(param_space_size);
        let mut index = vec![0usize; dimensions.len()];
        'outer: loop {
            let name = command_names
                .get(i)
                .or_else(|| command_names.get(0))
                .copied();
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

        Ok(commands)
    } else {
        let command_names = command_names.map_or(vec![], |names| names.collect::<Vec<&str>>());
        if command_names.len() > command_strings.len() {
            return Err(OptionsError::TooManyCommandNames(command_strings.len()).into());
        }

        let command_list = command_strings.collect::<Vec<&str>>();
        let mut commands = Vec::with_capacity(command_list.len());
        for (i, s) in command_list.iter().enumerate() {
            commands.push(Command::new(command_names.get(i).copied(), s));
        }
        Ok(commands)
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
        "par1",
        "a,b",
        "-L",
        "par2",
        "z,y",
        "echo {par1} {par2}",
        "printf '%s\n' {par1} {par2}",
    ]);
    let result = build_commands(&matches).unwrap();

    // Iteration order: command list first, then parameters in listed order (here, "par1" before
    // "par2", which is distinct from their sorted order), with parameter values in listed order.
    let pv = |s: &str| ParameterValue::Text(s.to_string());
    let cmd = |cmd: usize, par1: &str, par2: &str| {
        let expression = ["echo {par1} {par2}", "printf '%s\n' {par1} {par2}"][cmd];
        let params = vec![("par1", pv(par1)), ("par2", pv(par2))];
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
    let commands = build_commands(&matches).unwrap();
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
    let commands = build_commands(&matches).unwrap();
    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].get_name(), "name-1");
    assert_eq!(commands[1].get_name(), "name-2");
    assert_eq!(commands[0].get_shell_command(), "echo 1");
    assert_eq!(commands[1].get_shell_command(), "echo 2");
}
