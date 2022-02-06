use crate::command::Commands;
use crate::export::ExportManager;
use crate::options::{Options, OutputStyleOption};

use super::{mean_shell_spawning_time, relative_speed, result::BenchmarkResult, run_benchmark};

use anyhow::{bail, Result};
use colored::*;

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

pub fn run_benchmarks_and_print_comparison(
    commands: &Commands,
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
