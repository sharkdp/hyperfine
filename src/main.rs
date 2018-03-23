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
use std::error::Error;
use std::io;

use atty::Stream;
use colored::*;
use clap::{App, AppSettings, Arg};

mod hyperfine;

use hyperfine::internal::{write_benchmark_comparison, CmdFailureAction, HyperfineOptions,
                          OutputStyleOption};
use hyperfine::benchmark::{mean_shell_spawning_time, run_benchmark};
use hyperfine::export::{ExportEntry, ExportManager, ExportType};

/// Print error message to stderr and terminate
pub fn error(message: &str) -> ! {
    eprintln!("{} {}", "Error:".red(), message);
    std::process::exit(1);
}

/// Runs the benchmark for the given commands
fn run(commands: &Vec<&str>, options: &HyperfineOptions) -> io::Result<Vec<ExportEntry>> {
    let shell_spawning_time = mean_shell_spawning_time(&options.output_style)?;

    let mut timing_results = vec![];

    // Run the benchmarks
    for (num, cmd) in commands.iter().enumerate() {
        timing_results.push(run_benchmark(num, cmd, shell_spawning_time, options)?);
    }

    Ok(timing_results)
}

fn main() {
    // Process command line options
    let mut options = HyperfineOptions::default();

    let clap_color_setting = if atty::is(Stream::Stdout) {
        AppSettings::ColoredHelp
    } else {
        AppSettings::ColorNever
    };

    let matches = App::new("hyperfine")
        .version(crate_version!())
        .setting(clap_color_setting)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::UnifiedHelpMessage)
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
                .help(&format!(
                    "Perform at least NUM runs for each command (default: {}).",
                    options.min_runs
                )),
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
            Arg::with_name("style")
                .long("style")
                .short("s")
                .takes_value(true)
                .value_name("TYPE")
                .possible_values(&["auto", "basic", "full", "nocolor"])
                .help(
                    "Set output style type (default: auto). Set this to 'basic' to disable output \
                     coloring and interactive elements. Set it to 'full' to enable all effects \
                     even if no interactive terminal was detected. Setting to 'nocolor' maintains \
                     all advanced output, but uses no colorization.",
                ),
        )
        .arg(
            Arg::with_name("ignore-failure")
                .long("ignore-failure")
                .short("i")
                .help("Ignore non-zero exit codes."),
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
            Arg::with_name("compare")
                .long("compare")
                .short("c")
                .takes_value(false)
                .help("Display a comparison between two or more commands"),
        )
        .help_message("Print this help message.")
        .version_message("Show version information.")
        .get_matches();

    let compare_runs = matches.is_present("compare");
    let commands = matches.values_of("command").unwrap().collect::<Vec<&str>>();

    if compare_runs && commands.len() < 2 {
        error("Compare not applicable with fewer than two commands");
    }

    let str_to_u64 = |n| u64::from_str_radix(n, 10).ok();

    options.warmup_count = matches
        .value_of("warmup")
        .and_then(&str_to_u64)
        .unwrap_or(0);

    if let Some(min_runs) = matches.value_of("min-runs").and_then(&str_to_u64) {
        // we need at least two runs to compute a variance
        options.min_runs = cmp::max(2, min_runs);
    }

    options.preparation_command = matches.value_of("prepare").map(String::from);

    options.output_style = match matches.value_of("style") {
        Some("full") => OutputStyleOption::Full,
        Some("basic") => OutputStyleOption::Basic,
        Some("nocolor") => OutputStyleOption::NoColor,
        _ => if atty::is(Stream::Stdout) {
            OutputStyleOption::Full
        } else {
            OutputStyleOption::Basic
        },
    };

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

    // We default Windows to NoColor if full had been specified.
    if cfg!(windows) && options.output_style == OutputStyleOption::Full {
        options.output_style = OutputStyleOption::NoColor;
    }

    if options.output_style != OutputStyleOption::Full {
        colored::control::set_override(false);
    }

    if matches.is_present("ignore-failure") {
        options.failure_action = CmdFailureAction::Ignore;
    }

    let res = run(&commands, &options);

    match res {
        Ok(timing_results) => {
            if compare_runs {
                write_benchmark_comparison(&timing_results);
            }
            let ans = export_manager.write_results(timing_results);
            if let Err(e) = ans {
                error(&format!(
                    "The following error occured while exporting: {}",
                    e.description()
                ));
            }
        }
        Err(e) => error(e.description()),
    }
}
