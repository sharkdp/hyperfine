extern crate atty;

#[macro_use]
extern crate clap;
extern crate colored;
extern crate indicatif;
extern crate libc;
extern crate statistical;

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

use hyperfine::internal::{CmdFailureAction, HyperfineOptions, OutputStyleOption};
use hyperfine::benchmark::{mean_shell_spawning_time, run_benchmark};

/// Print error message to stderr and terminate
pub fn error(message: &str, style: &OutputStyleOption) -> ! {
    let error_title = match style {
        &OutputStyleOption::Basic => "Error:".white(),
        &OutputStyleOption::Full => "Error:".red(),
    };

    eprintln!("{} {}", error_title, message);
    std::process::exit(1);
}

/// Runs the benchmark for the given commands
fn run(commands: &Vec<&str>, options: &HyperfineOptions) -> io::Result<()> {
    let shell_spawning_time = mean_shell_spawning_time(&options.output_style)?;

    // Run the benchmarks
    for (num, cmd) in commands.iter().enumerate() {
        run_benchmark(num, cmd, shell_spawning_time, options)?;
    }

    Ok(())
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
                .possible_values(&["auto", "basic", "full"])
                .help(
                    "Set output style type. If set to 'basic', all colors and special \
                     formatting will be disabled. If set to 'auto' when output target is not \
                     a TTY, 'basic' is used (default: auto).",
                ),
        )
        .arg(
            Arg::with_name("ignore-failure")
                .long("ignore-failure")
                .short("i")
                .help("Ignore non-zero exit codes."),
        )
        .help_message("Print this help message.")
        .version_message("Show version information.")
        .get_matches();

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
        _ => if atty::is(Stream::Stdout) {
            OutputStyleOption::Full
        } else {
            OutputStyleOption::Basic
        },
    };

    if matches.is_present("ignore-failure") {
        options.failure_action = CmdFailureAction::Ignore;
    }

    let commands = matches.values_of("command").unwrap().collect();
    let res = run(&commands, &options);

    if let Err(e) = res {
        error(e.description(), &options.output_style);
    }
}
