extern crate ansi_term;
extern crate clap;
extern crate indicatif;

use std::process::{Command, Stdio};
use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};
use ansi_term::Colour::{Cyan, Green, Red};
use clap::{App, AppSettings, Arg};

struct CmdResult {
    duration_sec: f64,
    success: bool,
}

impl CmdResult {
    fn new(duration_sec: f64, success: bool) -> CmdResult {
        CmdResult {
            duration_sec,
            success,
        }
    }
}

fn run_shell_command(shell_cmd: &str) -> CmdResult {
    let start = Instant::now();

    let status = Command::new("sh")
        .arg("-c")
        .arg(shell_cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("failed to execute process");

    let duration = start.elapsed();

    let duration_sec = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;

    CmdResult::new(duration_sec, status.success())
}

fn main() {
    let matches = App::new("hyperfine")
        .global_settings(&[AppSettings::ColoredHelp])
        .version("0.1")
        .about("A command-line benchmarking tool")
        .author("David Peter <mail@david-peter.de>")
        .arg(
            Arg::with_name("command")
                .help("Command to benchmark")
                .required(true)
                .multiple(true),
        )
        .get_matches();

    let min_time_sec = 5.0;
    let min_runs = 10;

    let commands = matches.values_of("command").unwrap();
    for cmd in commands {
        println!("Command: {}", Cyan.paint(cmd));
        println!();

        let mut results = vec![];

        // Set up progress bar
        let bar = ProgressBar::new(min_runs);
        bar.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template(" {spinner} {msg:<30} {wide_bar} ETA {eta_precise}"),
        );
        bar.enable_steady_tick(80);
        bar.set_message("Initial time measurement");

        // Initial run
        let res = run_shell_command(cmd);

        let runs_in_min_time = (min_time_sec / res.duration_sec) as u64;

        let count = if runs_in_min_time >= min_runs {
            runs_in_min_time
        } else {
            min_runs
        };

        results.push(res);

        bar.set_length(count);
        bar.set_message("Collecting statistics");

        for _ in 1..count {
            bar.inc(1);
            let res = run_shell_command(cmd);
            results.push(res);
        }
        bar.finish_and_clear();

        let t_sum: f64 = results.iter().map(|r| r.duration_sec).sum();
        let t_mean = t_sum / (results.len() as f64);

        let t2_sum: f64 = results.iter().map(|r| r.duration_sec.powi(2)).sum();
        let t2_mean = t2_sum / (results.len() as f64);

        let stddev = (t2_mean - t_mean.powi(2)).sqrt();

        let time_fmt = format!("({:.3} ± {:.3}) s", t_mean, stddev);

        println!("  Time: {}", Green.paint(time_fmt));

        if !results.iter().all(|r| r.success) {
            println!(
                "  {}: Program returned non-zero exit status",
                Red.paint("Warning")
            );
        };

        println!();
    }
}
