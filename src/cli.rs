use std::ffi::OsString;

use clap::{crate_version, AppSettings, Arg, ArgMatches, Command};

pub fn get_cli_arguments<'a, I, T>(args: I) -> ArgMatches
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone + 'a,
{
    let command = build_command();
    command.get_matches_from(args)
}

/// Build the clap command for parsing command line arguments
fn build_command() -> Command<'static> {
    Command::new("hyperfine")
        .version(crate_version!())
        .setting(AppSettings::DeriveDisplayOrder)
        .next_line_help(true)
        .hide_possible_values(true)
        .max_term_width(90)
        .about("A command-line benchmarking tool.")
        .arg(
            Arg::new("command")
                .help("The command to benchmark. This can be the name of an executable, a command \
                       line like \"grep -i todo\" or a shell command like \"sleep 0.5 && echo test\". \
                       The latter is only available if the shell is not explicitly disabled via \
                       '--shell=none'. If multiple commands are given, hyperfine will show a \
                       comparison of the respective runtimes.")
                .required(true)
                .multiple_occurrences(true)
                .forbid_empty_values(true),
        )
        .arg(
            Arg::new("warmup")
                .long("warmup")
                .short('w')
                .takes_value(true)
                .value_name("NUM")
                .help(
                    "Perform NUM warmup runs before the actual benchmark. This can be used \
                     to fill (disk) caches for I/O-heavy programs.",
                ),
        )
        .arg(
            Arg::new("min-runs")
                .long("min-runs")
                .short('m')
                .takes_value(true)
                .value_name("NUM")
                .help("Perform at least NUM runs for each command (default: 10)."),
        )
        .arg(
            Arg::new("max-runs")
                .long("max-runs")
                .short('M')
                .takes_value(true)
                .value_name("NUM")
                .help("Perform at most NUM runs for each command. By default, there is no limit."),
        )
        .arg(
            Arg::new("runs")
                .long("runs")
                .conflicts_with_all(&["max-runs", "min-runs"])
                .short('r')
                .takes_value(true)
                .value_name("NUM")
                .help("Perform exactly NUM runs for each command. If this option is not specified, \
                       hyperfine automatically determines the number of runs."),
        )
        .arg(
            Arg::new("setup")
                .long("setup")
                .short('s')
                .takes_value(true)
                .multiple_occurrences(true)
                .number_of_values(1)
                .value_name("CMD")
                .help(
                    "Execute CMD before each set of timing runs. This is useful for \
                     compiling your software with the provided parameters, or to do any \
                     other work that should happen once before a series of benchmark runs, \
                     not every time as would happen with the --prepare option."
                ),
        )
        .arg(
            Arg::new("prepare")
                .long("prepare")
                .short('p')
                .takes_value(true)
                .multiple_occurrences(true)
                .number_of_values(1)
                .value_name("CMD")
                .help(
                    "Execute CMD before each timing run. This is useful for \
                     clearing disk caches, for example.\nThe --prepare option can \
                     be specified once for all commands or multiple times, once for \
                     each command. In the latter case, each preparation command will \
                     be run prior to the corresponding benchmark command.",
                ),
        )
        .arg(
            Arg::new("cleanup")
                .long("cleanup")
                .short('c')
                .takes_value(true)
                .value_name("CMD")
                .help(
                    "Execute CMD after the completion of all benchmarking \
                     runs for each individual command to be benchmarked. \
                     This is useful if the commands to be benchmarked produce \
                     artifacts that need to be cleaned up."
                ),
        )
        .arg(
            Arg::new("parameter-scan")
                .long("parameter-scan")
                .short('P')
                .takes_value(true)
                .allow_hyphen_values(true)
                .value_names(&["VAR", "MIN", "MAX"])
                .help(
                    "Perform benchmark runs for each value in the range MIN..MAX. Replaces the \
                     string '{VAR}' in each command by the current parameter value.\n\n  \
                     Example:  hyperfine -P threads 1 8 'make -j {threads}'\n\n\
                     This performs benchmarks for 'make -j 1', 'make -j 2', …, 'make -j 8'.\n\n\
                     To have the value increase following different patterns, use shell arithmetics.\n\n  \
                     Example: hyperfine -P size 0 3 'sleep $((2**{size}))'\n\n\
                     This performs benchmarks with power of 2 increases: 'sleep 1', 'sleep 2', 'sleep 4', …\n\
                     The exact syntax may vary depending on your shell and OS."
                ),
        )
        .arg(
            Arg::new("parameter-step-size")
                .long("parameter-step-size")
                .short('D')
                .takes_value(true)
                .value_names(&["DELTA"])
                .requires("parameter-scan")
                .help(
                    "This argument requires --parameter-scan to be specified as well. \
                     Traverse the range MIN..MAX in steps of DELTA.\n\n  \
                     Example:  hyperfine -P delay 0.3 0.7 -D 0.2 'sleep {delay}'\n\n\
                     This performs benchmarks for 'sleep 0.3', 'sleep 0.5' and 'sleep 0.7'.",
                ),
        )
        .arg(
            Arg::new("parameter-list")
                .long("parameter-list")
                .short('L')
                .takes_value(true)
                .multiple_occurrences(true)
                .allow_hyphen_values(true)
                .value_names(&["VAR", "VALUES"])
                .conflicts_with_all(&["parameter-scan", "parameter-step-size"])
                .help(
                    "Perform benchmark runs for each value in the comma-separated list VALUES. \
                     Replaces the string '{VAR}' in each command by the current parameter value\
                     .\n\nExample:  hyperfine -L compiler gcc,clang '{compiler} -O2 main.cpp'\n\n\
                     This performs benchmarks for 'gcc -O2 main.cpp' and 'clang -O2 main.cpp'.\n\n\
                     The option can be specified multiple times to run benchmarks for all \
                     possible parameter combinations.\n"
                ),
        )
        .arg(
            Arg::new("style")
                .long("style")
                .takes_value(true)
                .value_name("TYPE")
                .possible_values(&["auto", "basic", "full", "nocolor", "color", "none"])
                .help(
                    "Set output style type (default: auto). Set this to 'basic' to disable output \
                     coloring and interactive elements. Set it to 'full' to enable all effects \
                     even if no interactive terminal was detected. Set this to 'nocolor' to \
                     keep the interactive output without any colors. Set this to 'color' to keep \
                     the colors without any interactive output. Set this to 'none' to disable all \
                     the output of the tool.",
                ),
        )
        .arg(
            Arg::new("shell")
                .long("shell")
                .short('S')
                .takes_value(true)
                .value_name("SHELL")
                .overrides_with("shell")
                .help("Set the shell to use for executing benchmarked commands. This can be the \
                       name or the path to the shell executable, or a full command line \
                       like \"bash --norc\". It can also be set to \"default\" to explicitly select \
                       the default shell on this platform. Finally, this can also be set to \
                       \"none\" to disable the shell. In this case, commands will be executed \
                       directly. They can still have arguments, but more complex things like \
                       \"sleep 0.1; sleep 0.2\" are not possible without a shell.")
        )
        .arg(
            Arg::new("no-shell")
                .short('N')
                .conflicts_with_all(&["shell", "debug-mode"])
                .help("An alias for '--shell=none'.")
        )
        .arg(
            Arg::new("ignore-failure")
                .long("ignore-failure")
                .short('i')
                .help("Ignore non-zero exit codes of the benchmarked programs."),
        )
        .arg(
            Arg::new("time-unit")
                .long("time-unit")
                .short('u')
                .takes_value(true)
                .value_name("UNIT")
                .possible_values(&["millisecond", "second"])
                .help("Set the time unit to be used. Possible values: millisecond, second."),
        )
        .arg(
            Arg::new("export-asciidoc")
                .long("export-asciidoc")
                .takes_value(true)
                .value_name("FILE")
                .help("Export the timing summary statistics as an AsciiDoc table to the given FILE."),
        )
        .arg(
            Arg::new("export-csv")
                .long("export-csv")
                .takes_value(true)
                .value_name("FILE")
                .help("Export the timing summary statistics as CSV to the given FILE. If you need \
                       the timing results for each individual run, use the JSON export format."),
        )
        .arg(
            Arg::new("export-json")
                .long("export-json")
                .takes_value(true)
                .value_name("FILE")
                .help("Export the timing summary statistics and timings of individual runs as JSON to the given FILE."),
        )
        .arg(
            Arg::new("export-markdown")
                .long("export-markdown")
                .takes_value(true)
                .value_name("FILE")
                .help("Export the timing summary statistics as a Markdown table to the given FILE."),
        )
        .arg(
            Arg::new("show-output")
                .long("show-output")
                .conflicts_with("style")
                .help(
                    "Print the stdout and stderr of the benchmark instead of suppressing it. \
                     This will increase the time it takes for benchmarks to run, \
                     so it should only be used for debugging purposes or \
                     when trying to benchmark output speed.",
                ),
        )
        .arg(
            Arg::new("command-name")
                .long("command-name")
                .short('n')
                .takes_value(true)
                .multiple_occurrences(true)
                .number_of_values(1)
                .value_name("NAME")
                .help("Give a meaningful name to a command. This can be specified multiple times \
                       if several commands are benchmarked."),
        )
        .arg(
            Arg::new("debug-mode")
            .long("debug-mode")
            .hide(true)
            .help("Enable debug mode which does not actually run commands, but returns fake times when the command is 'sleep <time>'")
        )
}

#[test]
fn verify_app() {
    build_command().debug_assert();
}
