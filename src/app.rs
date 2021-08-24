use atty::Stream;
use clap::{crate_version, App, AppSettings, Arg, ArgMatches};
use std::ffi::OsString;

pub fn get_arg_matches<I, T>(args: I) -> ArgMatches<'static>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let app = build_app();
    app.get_matches_from(args)
}

/// Build the clap app for parsing command line arguments
fn build_app() -> App<'static, 'static> {
    let clap_color_setting = if atty::is(Stream::Stdout) {
        AppSettings::ColoredHelp
    } else {
        AppSettings::ColorNever
    };

    App::new("hyperfine")
        .version(crate_version!())
        .setting(clap_color_setting)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::NextLineHelp)
        .setting(AppSettings::HidePossibleValuesInHelp)
        .max_term_width(90)
        .about("A command-line benchmarking tool.")
        .arg(
            Arg::with_name("command")
                .help("Command to benchmark")
                .required(true)
                .multiple(true)
                .empty_values(false),
        )
        .arg(
            Arg::with_name("warmup")
                .long("warmup")
                .short("w")
                .takes_value(true)
                .value_name("NUM")
                .help(
                    "Perform NUM warmup runs before the actual benchmark. This can be used \
                     to fill (disk) caches for I/O-heavy programs.",
                ),
        )
        .arg(
            Arg::with_name("min-runs")
                .long("min-runs")
                .short("m")
                .takes_value(true)
                .value_name("NUM")
                .help("Perform at least NUM runs for each command (default: 10)."),
        )
        .arg(
            Arg::with_name("max-runs")
                .long("max-runs")
                .short("M")
                .takes_value(true)
                .value_name("NUM")
                .help("Perform at most NUM runs for each command. By default, there is no limit."),
        )
        .arg(
            Arg::with_name("runs")
                .long("runs")
                .conflicts_with_all(&["max-runs", "min-runs"])
                .short("r")
                .takes_value(true)
                .value_name("NUM")
                .help("Perform exactly NUM runs for each command. If this option is not specified, \
                       hyperfine automatically determines the number of runs."),
        )
        .arg(
            Arg::with_name("prepare")
                .long("prepare")
                .short("p")
                .takes_value(true)
                .multiple(true)
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
            Arg::with_name("cleanup")
                .long("cleanup")
                .short("c")
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
            Arg::with_name("parameter-scan")
                .long("parameter-scan")
                .short("P")
                .takes_value(true)
                .allow_hyphen_values(true)
                .value_names(&["VAR", "MIN", "MAX"])
                .help(
                    "Perform benchmark runs for each value in the range MIN..MAX. Replaces the \
                     string '{VAR}' in each command by the current parameter value.\n\n  \
                     Example:  hyperfine -P threads 1 8 'make -j {threads}'\n\n\
                     This performs benchmarks for 'make -j 1', 'make -j 2', …, 'make -j 8'.",
                ),
        )
        .arg(
            Arg::with_name("parameter-step-size")
                .long("parameter-step-size")
                .short("D")
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
            Arg::with_name("parameter-list")
                .long("parameter-list")
                .short("L")
                .takes_value(true)
                .multiple(true)
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
            Arg::with_name("style")
                .long("style")
                .short("s")
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
            Arg::with_name("shell")
                .long("shell")
                .short("S")
                .takes_value(true)
                .value_name("SHELL")
                .overrides_with("shell")
                .help("Set the shell to use for executing benchmarked commands."),
        )
        .arg(
            Arg::with_name("ignore-failure")
                .long("ignore-failure")
                .short("i")
                .help("Ignore non-zero exit codes of the benchmarked programs."),
        )
        .arg(
            Arg::with_name("time-unit")
                .long("time-unit")
                .short("u")
                .takes_value(true)
                .value_name("UNIT")
                .possible_values(&["millisecond", "second"])
                .help("Set the time unit to be used. Possible values: millisecond, second."),
        )
        .arg(
            Arg::with_name("export-asciidoc")
                .long("export-asciidoc")
                .takes_value(true)
                .value_name("FILE")
                .help("Export the timing summary statistics as an AsciiDoc table to the given FILE."),
        )
        .arg(
            Arg::with_name("export-csv")
                .long("export-csv")
                .takes_value(true)
                .value_name("FILE")
                .help("Export the timing summary statistics as CSV to the given FILE. If you need \
                       the timing results for each individual run, use the JSON export format."),
        )
        .arg(
            Arg::with_name("export-json")
                .long("export-json")
                .takes_value(true)
                .value_name("FILE")
                .help("Export the timing summary statistics and timings of individual runs as JSON to the given FILE."),
        )
        .arg(
            Arg::with_name("export-markdown")
                .long("export-markdown")
                .takes_value(true)
                .value_name("FILE")
                .help("Export the timing summary statistics as a Markdown table to the given FILE."),
        )
        .arg(
            Arg::with_name("show-output")
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
            Arg::with_name("command-name")
                .long("command-name")
                .short("n")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1)
                .value_name("NAME")
                .help("Give a meaningful name to a command"),
        )
        .help_message("Print this help message.")
        .version_message("Show version information.")
}
