use anyhow::Result;

use crate::benchmark::benchmark_result::BenchmarkResult;
use crate::benchmark::executor::Executor;
use crate::benchmark::{Benchmark, BenchmarkState};
use crate::command::Command;
use crate::options::Options;

/// Run timing iterations for multiple commands in an interleaved fashion.
///
/// For each round, every command is executed once before moving on to the next round. This
/// preserves per-command statistics while ensuring that benchmark runs are spread across time
/// (useful for dependent pipelines and for mitigating temporal system load bias).
pub fn run_sequential_benchmarks<'a>(
    commands: &[(usize, &'a Command<'a>)],
    options: &'a Options,
    executor: &'a dyn Executor,
) -> Result<Vec<BenchmarkResult>> {
    let benchmarks: Vec<Benchmark<'a>> = commands
        .iter()
        .map(|(number, command)| Benchmark::new(*number, command, options, executor))
        .collect();

    let mut states: Vec<BenchmarkState> = (0..benchmarks.len())
        .map(|_| BenchmarkState::new())
        .collect();
    let mut run_counts = vec![0_u64; benchmarks.len()];

    for benchmark in &benchmarks {
        benchmark.print_header();
        benchmark.run_setup()?;
        benchmark.run_warmup()?;
    }

    for (index, benchmark) in benchmarks.iter().enumerate() {
        let overheads = benchmark.run_measured_iteration(0, &mut states[index])?;
        let first_run_time = states[index].times_real[0];
        run_counts[index] = benchmark.compute_run_count(first_run_time, overheads);
    }

    let max_runs = *run_counts.iter().max().unwrap_or(&0);

    for round in 1..max_runs {
        for (index, benchmark) in benchmarks.iter().enumerate() {
            if round < run_counts[index] {
                benchmark.run_measured_iteration(round, &mut states[index])?;
            }
        }
    }

    let mut results = Vec::with_capacity(benchmarks.len());
    for (benchmark, state) in benchmarks.iter().zip(states) {
        results.push(benchmark.finalize(state)?);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::get_cli_arguments;
    use crate::command::Commands;
    use crate::export::ExportManager;
    use crate::options::{ExecutorKind, Options};

    fn run_sequential(args: &[&str]) -> Result<Vec<BenchmarkResult>> {
        let args = [
            "hyperfine",
            "--debug-mode",
            "--style=none",
            "--run-sequentially",
        ]
        .iter()
        .chain(args);
        let cli_arguments = get_cli_arguments(args);
        let mut options = Options::from_cli_arguments(&cli_arguments)?;
        assert_eq!(options.executor_kind, ExecutorKind::Mock(None));

        let commands = Commands::from_cli_arguments(&cli_arguments)?;
        options.validate_against_command_list(&commands)?;

        let _export_manager = ExportManager::from_cli_arguments(
            &cli_arguments,
            options.time_unit,
            options.sort_order_exports,
        )?;

        let mut executor: Box<dyn Executor> =
            Box::new(crate::benchmark::executor::MockExecutor::new(None));
        executor.calibrate()?;

        let command_refs: Vec<(usize, _)> = commands
            .iter()
            .enumerate()
            .map(|(i, cmd)| (i, cmd))
            .collect();

        run_sequential_benchmarks(&command_refs, &options, executor.as_ref())
    }

    #[test]
    fn sequential_mode_interleaves_runs() -> Result<()> {
        let results = run_sequential(&["--runs=2", "sleep 0.123", "sleep 0.456"])?;

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].times.as_ref().unwrap().len(), 2);
        assert_eq!(results[1].times.as_ref().unwrap().len(), 2);
        assert!((results[0].mean - 0.123).abs() < 1e-9);
        assert!((results[1].mean - 0.456).abs() < 1e-9);

        Ok(())
    }
}
