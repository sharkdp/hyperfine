use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::parameter::tokenize::tokenize;
use crate::parameter::ParameterValue;
use crate::{
    error::{OptionsError, ParameterScanError},
    parameter::{
        range_step::{Numeric, RangeStep},
        ParameterNameAndValue,
    },
};

use clap::{parser::ValuesRef, ArgMatches};

use anyhow::{bail, Context, Result};
use rust_decimal::Decimal;

/// A command that should be benchmarked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command<'a> {
    /// The command name (without parameter substitution)
    name: Option<&'a str>,

    /// The command that should be executed (without parameter substitution)
    expression: &'a str,

    /// Zero or more parameter values.
    parameters: Vec<ParameterNameAndValue<'a>>,
}

impl<'a> Command<'a> {
    pub fn new(name: Option<&'a str>, expression: &'a str) -> Command<'a> {
        Command {
            name,
            expression,
            parameters: Vec::new(),
        }
    }

    pub fn new_parametrized(
        name: Option<&'a str>,
        expression: &'a str,
        parameters: impl IntoIterator<Item = ParameterNameAndValue<'a>>,
    ) -> Command<'a> {
        Command {
            name,
            expression,
            parameters: parameters.into_iter().collect(),
        }
    }

    pub fn get_name(&self) -> String {
        self.name.map_or_else(
            || self.get_command_line(),
            |name| self.replace_parameters_in(name),
        )
    }

    pub fn get_name_with_unused_parameters(&self) -> String {
        let parameters = self
            .get_unused_parameters()
            .fold(String::new(), |output, (parameter, value)| {
                output + &format!("{parameter} = {value}, ")
            });
        let parameters = parameters.trim_end_matches(", ");
        let parameters = if parameters.is_empty() {
            "".into()
        } else {
            format!(" ({parameters})")
        };

        format!("{}{}", self.get_name(), parameters)
    }

    pub fn get_command_line(&self) -> String {
        self.replace_parameters_in(self.expression)
    }

    pub fn get_command(&self) -> Result<std::process::Command> {
        let command_line = self.get_command_line();
        let mut tokens = shell_words::split(&command_line)
            .with_context(|| format!("Failed to parse command '{command_line}'"))?
            .into_iter();

        if let Some(program_name) = tokens.next() {
            let mut command_builder = std::process::Command::new(program_name);
            command_builder.args(tokens);
            Ok(command_builder)
        } else {
            bail!("Can not execute empty command")
        }
    }

    pub fn get_parameters(&self) -> &[(&'a str, ParameterValue)] {
        &self.parameters
    }

    pub fn get_unused_parameters(&self) -> impl Iterator<Item = &(&'a str, ParameterValue)> {
        self.parameters
            .iter()
            .filter(move |(parameter, _)| !self.expression.contains(&format!("{{{parameter}}}")))
    }

    fn replace_parameters_in(&self, original: &str) -> String {
        let mut result = String::new();
        let mut replacements = BTreeMap::<String, String>::new();
        for (param_name, param_value) in &self.parameters {
            replacements.insert(format!("{{{param_name}}}"), param_value.to_string());
        }
        let mut remaining = original;
        // Manually replace consecutive occurrences to avoid double-replacing: e.g.,
        //
        //     hyperfine -L foo 'a,{bar}' -L bar 'baz,quux' 'echo {foo} {bar}'
        //
        // should not ever run 'echo baz baz'. See `test_get_command_line_nonoverlapping`.
        'outer: while let Some(head) = remaining.chars().next() {
            for (k, v) in &replacements {
                if remaining.starts_with(k.as_str()) {
                    result.push_str(v);
                    remaining = &remaining[k.len()..];
                    continue 'outer;
                }
            }
            result.push(head);
            remaining = &remaining[head.len_utf8()..];
        }
        result
    }
}

/// A collection of commands that should be benchmarked
pub struct Commands<'a> {
    source: CommandSource<'a>,
    len: usize,
}

enum CommandSource<'a> {
    Materialized(Vec<Command<'a>>),
    ParameterFile(ParameterFileCommands<'a>),
}

struct ParameterFileCommands<'a> {
    parameter_name: &'a str,
    path: PathBuf,
    command_names: Vec<&'a str>,
    command_strings: Vec<&'a str>,
}

impl<'a> Commands<'a> {
    pub fn from_cli_arguments(matches: &'a ArgMatches) -> Result<Commands<'a>> {
        let command_names = matches.get_many::<String>("command-name");
        let command_strings = matches
            .get_many::<String>("command")
            .unwrap_or_default()
            .map(|v| v.as_str())
            .collect::<Vec<_>>();

        if let Some(args) = matches.get_many::<String>("parameter-scan") {
            let step_size = matches
                .get_one::<String>("parameter-step-size")
                .map(|s| s.as_str());
            let commands =
                Self::get_parameter_scan_commands(command_names, command_strings, args, step_size)?;
            let len = commands.len();
            Ok(Self {
                source: CommandSource::Materialized(commands),
                len,
            })
        } else if let Some(mut args) = matches.get_many::<String>("parameter-file") {
            let parameter_name = args.next().unwrap().as_str();
            let path = PathBuf::from(args.next().unwrap().as_str());
            let line_count = Self::count_parameter_file_lines(&path)?;
            let len = command_strings
                .len()
                .checked_mul(line_count)
                .ok_or_else(|| {
                    anyhow::anyhow!("The parameter file produces too many benchmark commands")
                })?;
            let command_names = command_names.map_or(vec![], |names| {
                names.map(|v| v.as_str()).collect::<Vec<_>>()
            });

            // `--command-name` should appear exactly once or exactly B times,
            // where B is the total number of benchmarks.
            let command_name_count = command_names.len();
            if command_name_count > 1 && command_name_count != len {
                return Err(
                    OptionsError::UnexpectedCommandNameCount(command_name_count, len).into(),
                );
            }

            Ok(Self {
                source: CommandSource::ParameterFile(ParameterFileCommands {
                    parameter_name,
                    path,
                    command_names,
                    command_strings,
                }),
                len,
            })
        } else if let Some(args) = matches.get_many::<String>("parameter-list") {
            let command_names = command_names.map_or(vec![], |names| {
                names.map(|v| v.as_str()).collect::<Vec<_>>()
            });
            let args: Vec<_> = args.map(|v| v.as_str()).collect::<Vec<_>>();
            let param_names_and_values: Vec<(&str, Vec<String>)> = args
                .chunks_exact(2)
                .map(|pair| {
                    let name = pair[0];
                    let list_str = pair[1];
                    (name, tokenize(list_str))
                })
                .collect();
            {
                let duplicates =
                    Self::find_duplicates(param_names_and_values.iter().map(|(name, _)| *name));
                if !duplicates.is_empty() {
                    bail!("Duplicate parameter names: {}", &duplicates.join(", "));
                }
            }

            let dimensions: Vec<usize> = std::iter::once(command_strings.len())
                .chain(
                    param_names_and_values
                        .iter()
                        .map(|(_, values)| values.len()),
                )
                .collect();
            let param_space_size = dimensions.iter().product();
            if param_space_size == 0 {
                return Ok(Self {
                    source: CommandSource::Materialized(Vec::new()),
                    len: 0,
                });
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
                    .or_else(|| command_names.first())
                    .copied();
                i += 1;

                let (command_index, params_indices) = index.split_first().unwrap();
                let parameters: Vec<_> = param_names_and_values
                    .iter()
                    .zip(params_indices)
                    .map(|((name, values), i)| (*name, ParameterValue::Text(values[*i].clone())))
                    .collect();
                commands.push(Command::new_parametrized(
                    name,
                    command_strings[*command_index],
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

            let len = commands.len();
            Ok(Self {
                source: CommandSource::Materialized(commands),
                len,
            })
        } else {
            let command_names = command_names.map_or(vec![], |names| {
                names.map(|v| v.as_str()).collect::<Vec<_>>()
            });
            if command_names.len() > command_strings.len() {
                return Err(OptionsError::TooManyCommandNames(command_strings.len()).into());
            }

            let mut commands = Vec::with_capacity(command_strings.len());
            for (i, s) in command_strings.iter().enumerate() {
                commands.push(Command::new(command_names.get(i).copied(), s));
            }
            let len = commands.len();
            Ok(Self {
                source: CommandSource::Materialized(commands),
                len,
            })
        }
    }

    /// Calls `f` for each command without materializing parameter-file values in memory.
    pub fn try_for_each<F>(&self, mut f: F) -> Result<()>
    where
        F: FnMut(&Command<'a>) -> Result<()>,
    {
        match &self.source {
            CommandSource::Materialized(commands) => {
                for command in commands {
                    f(command)?;
                }
            }
            CommandSource::ParameterFile(parameter_file) => {
                let file = File::open(&parameter_file.path).with_context(|| {
                    format!(
                        "Failed to open parameter file '{}'",
                        parameter_file.path.display()
                    )
                })?;
                let mut benchmark_index = 0;

                for line in BufReader::new(file).lines() {
                    let line = line.with_context(|| {
                        format!(
                            "Failed to read parameter file '{}'",
                            parameter_file.path.display()
                        )
                    })?;

                    for command_string in &parameter_file.command_strings {
                        let name = parameter_file
                            .command_names
                            .get(benchmark_index)
                            .or_else(|| parameter_file.command_names.first())
                            .copied();
                        let command = Command::new_parametrized(
                            name,
                            command_string,
                            [(
                                parameter_file.parameter_name,
                                ParameterValue::Text(line.clone()),
                            )],
                        );
                        f(&command)?;
                        benchmark_index += 1;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn num_commands(&self, has_reference_command: bool) -> usize {
        self.len + if has_reference_command { 1 } else { 0 }
    }

    fn count_parameter_file_lines(path: &Path) -> Result<usize> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open parameter file '{}'", path.display()))?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        let mut count = 0usize;

        loop {
            buffer.clear();
            if reader
                .read_until(b'\n', &mut buffer)
                .with_context(|| format!("Failed to read parameter file '{}'", path.display()))?
                == 0
            {
                break;
            }
            count = count
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("The parameter file contains too many lines"))?;
        }

        Ok(count)
    }

    #[cfg(test)]
    fn materialized(&self) -> &[Command<'a>] {
        match &self.source {
            CommandSource::Materialized(commands) => commands,
            CommandSource::ParameterFile(_) => {
                panic!("parameter-file commands are generated lazily")
            }
        }
    }

    /// Finds all the strings that appear multiple times in the input iterator, returning them in
    /// sorted order. If no string appears more than once, the result is an empty vector.
    fn find_duplicates<'b, I: IntoIterator<Item = &'b str>>(i: I) -> Vec<&'b str> {
        let mut counts = BTreeMap::<&'b str, usize>::new();
        for s in i {
            *counts.entry(s).or_default() += 1;
        }
        counts
            .into_iter()
            .filter_map(|(k, n)| if n > 1 { Some(k) } else { None })
            .collect()
    }

    fn build_parameter_scan_commands<'b, T: Numeric>(
        param_name: &'b str,
        param_min: T,
        param_max: T,
        step: T,
        command_names: Vec<&'b str>,
        command_strings: Vec<&'b str>,
    ) -> Result<Vec<Command<'b>>, ParameterScanError> {
        let param_range = RangeStep::new(param_min, param_max, step)?;
        let command_name_count = command_names.len();

        let mut i = 0;
        let mut commands = vec![];
        for value in param_range {
            for cmd in &command_strings {
                let name = command_names
                    .get(i)
                    .or_else(|| command_names.first())
                    .copied();
                commands.push(Command::new_parametrized(
                    name,
                    cmd,
                    vec![(param_name, ParameterValue::Numeric(value.into()))],
                ));
                i += 1;
            }
        }

        // `--command-name` should appear exactly once or exactly B times,
        // where B is the total number of benchmarks.
        let command_count = commands.len();
        if command_name_count > 1 && command_name_count != command_count {
            return Err(ParameterScanError::UnexpectedCommandNameCount(
                command_name_count,
                command_count,
            ));
        }

        Ok(commands)
    }

    fn get_parameter_scan_commands<'b>(
        command_names: Option<ValuesRef<'b, String>>,
        command_strings: Vec<&'b str>,
        mut vals: ValuesRef<'b, String>,
        step: Option<&str>,
    ) -> Result<Vec<Command<'b>>, ParameterScanError> {
        let command_names = command_names.map_or(vec![], |names| {
            names.map(|v| v.as_str()).collect::<Vec<_>>()
        });
        let param_name = vals.next().unwrap().as_str();
        let param_min = vals.next().unwrap().as_str();
        let param_max = vals.next().unwrap().as_str();

        // attempt to parse as integers
        if let (Ok(param_min), Ok(param_max), Ok(step)) = (
            param_min.parse::<i32>(),
            param_max.parse::<i32>(),
            step.unwrap_or("1").parse::<i32>(),
        ) {
            return Self::build_parameter_scan_commands(
                param_name,
                param_min,
                param_max,
                step,
                command_names,
                command_strings,
            );
        }

        // try parsing them as decimals
        let param_min = Decimal::from_str(param_min)?;
        let param_max = Decimal::from_str(param_max)?;

        if step.is_none() {
            return Err(ParameterScanError::StepRequired);
        }

        let step = Decimal::from_str(step.unwrap())?;
        Self::build_parameter_scan_commands(
            param_name,
            param_min,
            param_max,
            step,
            command_names,
            command_strings,
        )
    }
}

#[test]
fn test_get_command_line_nonoverlapping() {
    let cmd = Command::new_parametrized(
        None,
        "echo {foo} {bar}",
        vec![
            ("foo", ParameterValue::Text("{bar} baz".into())),
            ("bar", ParameterValue::Text("quux".into())),
        ],
    );
    assert_eq!(cmd.get_command_line(), "echo {bar} baz quux");
}

#[test]
fn test_get_parameterized_command_name() {
    let cmd = Command::new_parametrized(
        Some("name-{bar}-{foo}"),
        "echo {foo} {bar}",
        vec![
            ("foo", ParameterValue::Text("baz".into())),
            ("bar", ParameterValue::Text("quux".into())),
        ],
    );
    assert_eq!(cmd.get_name(), "name-quux-baz");
}

impl fmt::Display for Command<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_command_line())
    }
}

#[test]
fn test_build_commands_cross_product() {
    use crate::cli::get_cli_arguments;

    let matches = get_cli_arguments(vec![
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
    let commands = Commands::from_cli_arguments(&matches).unwrap();
    let result = commands.materialized();

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
    use crate::cli::get_cli_arguments;

    let matches = get_cli_arguments(vec![
        "hyperfine",
        "echo {foo}",
        "--parameter-list",
        "foo",
        "1,2",
        "--command-name",
        "name-{foo}",
    ]);
    let command_source = Commands::from_cli_arguments(&matches).unwrap();
    let commands = command_source.materialized();
    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].get_name(), "name-1");
    assert_eq!(commands[1].get_name(), "name-2");
    assert_eq!(commands[0].get_command_line(), "echo 1");
    assert_eq!(commands[1].get_command_line(), "echo 2");
}

#[test]
fn test_build_parameter_file_commands_lazily() {
    use crate::cli::get_cli_arguments;
    use std::io::Write;

    let mut parameters = tempfile::NamedTempFile::new().unwrap();
    parameters.write_all(b"alpha\r\nbeta\nlast").unwrap();

    let matches = get_cli_arguments(vec![
        "hyperfine",
        "echo {value}",
        "print {value}",
        "--parameter-file",
        "value",
        parameters.path().to_str().unwrap(),
    ]);
    let commands = Commands::from_cli_arguments(&matches).unwrap();
    assert_eq!(commands.num_commands(false), 6);

    let mut command_lines = Vec::new();
    commands
        .try_for_each(|command| {
            command_lines.push(command.get_command_line());
            Ok(())
        })
        .unwrap();

    assert_eq!(
        command_lines,
        [
            "echo alpha",
            "print alpha",
            "echo beta",
            "print beta",
            "echo last",
            "print last",
        ]
    );
}

#[test]
fn test_parameter_file_large_input_is_processed_line_by_line() {
    use crate::cli::get_cli_arguments;
    use std::io::Write;

    let mut parameters = tempfile::NamedTempFile::new().unwrap();
    for i in 0..10_000 {
        writeln!(parameters, "{i}").unwrap();
    }

    let matches = get_cli_arguments(vec![
        "hyperfine",
        "echo {value}",
        "--parameter-file",
        "value",
        parameters.path().to_str().unwrap(),
    ]);
    let commands = Commands::from_cli_arguments(&matches).unwrap();
    assert_eq!(commands.num_commands(false), 10_000);

    let mut seen = 0;
    commands
        .try_for_each(|command| {
            if seen == 0 {
                assert_eq!(command.get_command_line(), "echo 0");
            }
            if seen == 9_999 {
                assert_eq!(command.get_command_line(), "echo 9999");
            }
            seen += 1;
            Ok(())
        })
        .unwrap();
    assert_eq!(seen, 10_000);
}

#[test]
fn test_build_parameter_scan_commands() {
    use crate::cli::get_cli_arguments;
    let matches = get_cli_arguments(vec![
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
    let command_source = Commands::from_cli_arguments(&matches).unwrap();
    let commands = command_source.materialized();
    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].get_name(), "name-1");
    assert_eq!(commands[1].get_name(), "name-2");
    assert_eq!(commands[0].get_command_line(), "echo 1");
    assert_eq!(commands[1].get_command_line(), "echo 2");
}

#[test]
fn test_build_parameter_scan_commands_named() {
    use crate::cli::get_cli_arguments;
    let matches = get_cli_arguments(vec![
        "hyperfine",
        "echo {val}",
        "sleep {val}",
        "--parameter-scan",
        "val",
        "1",
        "2",
        "--parameter-step-size",
        "1",
        "--command-name",
        "echo-1",
        "--command-name",
        "sleep-1",
        "--command-name",
        "echo-2",
        "--command-name",
        "sleep-2",
    ]);
    let command_source = Commands::from_cli_arguments(&matches).unwrap();
    let commands = command_source.materialized();
    assert_eq!(commands.len(), 4);
    assert_eq!(commands[0].get_name(), "echo-1");
    assert_eq!(commands[0].get_command_line(), "echo 1");
    assert_eq!(commands[1].get_name(), "sleep-1");
    assert_eq!(commands[1].get_command_line(), "sleep 1");
    assert_eq!(commands[2].get_name(), "echo-2");
    assert_eq!(commands[2].get_command_line(), "echo 2");
    assert_eq!(commands[3].get_name(), "sleep-2");
    assert_eq!(commands[3].get_command_line(), "sleep 2");
}

#[test]
fn test_parameter_scan_commands_int() {
    let commands = Commands::build_parameter_scan_commands(
        "val",
        1i32,
        7i32,
        3i32,
        vec![],
        vec!["echo {val}"],
    )
    .unwrap();
    assert_eq!(commands.len(), 3);
    assert_eq!(commands[2].get_name(), "echo 7");
    assert_eq!(commands[2].get_command_line(), "echo 7");
}

#[test]
fn test_parameter_scan_commands_decimal() {
    let param_min = Decimal::from_str("0").unwrap();
    let param_max = Decimal::from_str("1").unwrap();
    let step = Decimal::from_str("0.33").unwrap();

    let commands = Commands::build_parameter_scan_commands(
        "val",
        param_min,
        param_max,
        step,
        vec![],
        vec!["echo {val}"],
    )
    .unwrap();
    assert_eq!(commands.len(), 4);
    assert_eq!(commands[3].get_name(), "echo 0.99");
    assert_eq!(commands[3].get_command_line(), "echo 0.99");
}

#[test]
fn test_parameter_scan_commands_names() {
    let commands = Commands::build_parameter_scan_commands(
        "val",
        1i32,
        3i32,
        1i32,
        vec!["name-{val}"],
        vec!["echo {val}"],
    )
    .unwrap();
    assert_eq!(commands.len(), 3);
    let command_names = commands
        .iter()
        .map(|c| c.get_name())
        .collect::<Vec<String>>();
    assert_eq!(command_names, vec!["name-1", "name-2", "name-3"]);
}

#[test]
fn test_get_specified_command_names() {
    let commands = Commands::build_parameter_scan_commands(
        "val",
        1i32,
        3i32,
        1i32,
        vec!["name-a", "name-b", "name-c"],
        vec!["echo {val}"],
    )
    .unwrap();
    assert_eq!(commands.len(), 3);
    let command_names = commands
        .iter()
        .map(|c| c.get_name())
        .collect::<Vec<String>>();
    assert_eq!(command_names, vec!["name-a", "name-b", "name-c"]);
}

#[test]
fn test_different_command_name_count_with_parameters() {
    let result = Commands::build_parameter_scan_commands(
        "val",
        1i32,
        3i32,
        1i32,
        vec!["name-1", "name-2"],
        vec!["echo {val}"],
    );
    assert!(matches!(
        result.unwrap_err(),
        ParameterScanError::UnexpectedCommandNameCount(2, 3)
    ));
}
