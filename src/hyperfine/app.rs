use atty;
use atty::Stream;
use clap::{App, AppSettings, Arg, ArgMatches};
use std::ffi::OsString;

pub fn get_arg_matches<T>(args: T) -> ArgMatches<'static>
where
    T: Iterator<Item = OsString>,
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
                .help("Perform at most NUM runs for each command."),
        )
        .arg(
            Arg::with_name("runs")
                .long("runs")
                .conflicts_with_all(&["max-runs", "min-runs"])
                .short("r")
                .takes_value(true)
                .value_name("NUM")
                .help("Perform exactly NUM runs for each command."),
        )
        .arg(
            Arg::with_name("prepare")
                .long("prepare")
                .short("p")
                .takes_value(true)
                .value_name("CMD")
                .help(
                    "Execute CMD before each timing run. This is useful for \
                     clearing disk caches, for example.",
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
                     string '{VAR}' in each command by the current parameter value.",
                ),
        )
        .arg(
            Arg::with_name("style")
                .long("style")
                .short("s")
                .takes_value(true)
                .value_name("TYPE")
                .possible_values(&["auto", "basic", "full", "nocolor", "color"])
                .help(
                    "Set output style type (default: auto). Set this to 'basic' to disable output \
                     coloring and interactive elements. Set it to 'full' to enable all effects \
                     even if no interactive terminal was detected. Set this to 'nocolor' to \
                     keep the interactive output without any colors. Set this to 'color to' keep \
                     the colors without any interactive output.",
                ),
        )
        .arg(
            Arg::with_name("shell")
                .long("shell")
                .short("S")
                .takes_value(true)
                .value_name("SHELL")
                .overrides_with("shell")
                .help("Set the shell to use (default: sh) for executing benchmarked commands."),
        )
        .arg(
            Arg::with_name("ignore-failure")
                .long("ignore-failure")
                .short("i")
                .help("Ignore non-zero exit codes."),
        )
        .arg(
            Arg::with_name("time-unit")
                .long("time-unit")
                .short("u")
                .takes_value(true)
                .value_name("millisecond|second")
                .possible_values(&["millisecond", "second"])
                .help( "Set the time unit used for CLI output and Markdown export.",
                ),
        )
        .arg(
            Arg::with_name("export-csv")
                .long("export-csv")
                .takes_value(true)
                .value_name("FILE")
                .help("Export the timing results as CSV to the given FILE."),
        )
        .arg(
            Arg::with_name("export-json")
                .long("export-json")
                .takes_value(true)
                .value_name("FILE")
                .help("Export the timing results as JSON to the given FILE."),
        )
        .arg(
            Arg::with_name("export-markdown")
                .long("export-markdown")
                .takes_value(true)
                .value_name("FILE")
                .help("Export the timing results as a Markdown table to the given FILE."),
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
        .help_message("Print this help message.")
        .version_message("Show version information.")
}
