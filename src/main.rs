extern crate ansi_term;
extern crate clap;

use std::process::{Command, Stdio};
use std::time::Instant;
use std::io::{stdout, Write};

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

    let commands = matches.values_of("command").unwrap();
    for cmd in commands {
        println!("Command  '{}'", Cyan.paint(cmd));
        print!("Running benchmark ");

        let mut results = vec![];
        for _ in 1..10 {
            print!(".");
            let res = run_shell_command(cmd);
            results.push(res);
            let _ = stdout().flush();
        }
        println!(" done");

        let duration_sum: f64 = results.iter().map(|r| r.duration_sec).sum();
        let duration_mean = duration_sum / (results.len() as f64);
        let duration_str = format!("{:.3} s", duration_mean);
        println!("  Time: {:.3}", Green.paint(duration_str));

        if !results.iter().all(|r| r.success) {
            println!("{}", Red.paint("Warning: non-zero exit code"));
        };

        println!();
    }
}
