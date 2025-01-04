mod common;
use common::hyperfine;

use predicates::prelude::*;

/// Platform-specific I/O utility.
/// - On Unix-like systems, defaults to `cat`.
/// - On Windows, uses `findstr` as an alternative.
///   See: <https://superuser.com/questions/853580/real-windows-equivalent-to-cat-stdin>
const STDIN_READ_COMMAND: &str = if cfg!(windows) { "findstr x*" } else { "cat" };

pub fn hyperfine_debug() -> assert_cmd::Command {
    let mut cmd = hyperfine();
    cmd.arg("--debug-mode");
    cmd
}

#[test]
fn runs_successfully() {
    hyperfine()
        .arg("--runs=2")
        .arg("echo dummy benchmark")
        .assert()
        .success();
}

#[test]
fn one_run_is_supported() {
    hyperfine()
        .arg("--runs=1")
        .arg("echo dummy benchmark")
        .assert()
        .success();
}

#[test]
fn can_run_commands_without_a_shell() {
    hyperfine()
        .arg("--runs=1")
        .arg("--show-output")
        .arg("--shell=none")
        .arg("echo 'hello world' argument2")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world argument2"));
}

#[test]
fn fails_with_wrong_number_of_command_name_arguments() {
    hyperfine()
        .arg("--command-name=a")
        .arg("--command-name=b")
        .arg("echo a")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Too many --command-name options"));
}

#[test]
fn fails_with_wrong_number_of_prepare_options() {
    hyperfine()
        .arg("--runs=1")
        .arg("--prepare=echo a")
        .arg("--prepare=echo b")
        .arg("echo a")
        .arg("echo b")
        .assert()
        .success();

    hyperfine()
        .arg("--runs=1")
        .arg("--prepare=echo ref")
        .arg("--prepare=echo a")
        .arg("--prepare=echo b")
        .arg("--reference=echo ref")
        .arg("echo a")
        .arg("echo b")
        .assert()
        .success();

    hyperfine()
        .arg("--runs=1")
        .arg("--prepare=echo a")
        .arg("--prepare=echo b")
        .arg("echo a")
        .arg("echo b")
        .arg("echo c")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "The '--prepare' option has to be provided",
        ));

    hyperfine()
        .arg("--runs=1")
        .arg("--prepare=echo a")
        .arg("--prepare=echo b")
        .arg("--reference=echo ref")
        .arg("echo a")
        .arg("echo b")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "The '--prepare' option has to be provided",
        ));
}

#[test]
fn fails_with_wrong_number_of_conclude_options() {
    hyperfine()
        .arg("--runs=1")
        .arg("--conclude=echo a")
        .arg("--conclude=echo b")
        .arg("echo a")
        .arg("echo b")
        .assert()
        .success();

    hyperfine()
        .arg("--runs=1")
        .arg("--conclude=echo ref")
        .arg("--conclude=echo a")
        .arg("--conclude=echo b")
        .arg("--reference=echo ref")
        .arg("echo a")
        .arg("echo b")
        .assert()
        .success();

    hyperfine()
        .arg("--runs=1")
        .arg("--conclude=echo a")
        .arg("--conclude=echo b")
        .arg("echo a")
        .arg("echo b")
        .arg("echo c")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "The '--conclude' option has to be provided",
        ));

    hyperfine()
        .arg("--runs=1")
        .arg("--conclude=echo a")
        .arg("--conclude=echo b")
        .arg("--reference=echo ref")
        .arg("echo a")
        .arg("echo b")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "The '--conclude' option has to be provided",
        ));
}

#[test]
fn fails_with_duplicate_parameter_names() {
    hyperfine()
        .arg("--parameter-list")
        .arg("x")
        .arg("1,2,3")
        .arg("--parameter-list")
        .arg("x")
        .arg("a,b,c")
        .arg("echo test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Duplicate parameter names: x"));
}

#[test]
fn fails_for_unknown_command() {
    hyperfine()
        .arg("--runs=1")
        .arg("some-nonexisting-program-b5d9574198b7e4b12a71fa4747c0a577")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Command terminated with non-zero exit code",
        ));
}

#[test]
fn fails_for_unknown_command_without_shell() {
    hyperfine()
        .arg("--shell=none")
        .arg("--runs=1")
        .arg("some-nonexisting-program-b5d9574198b7e4b12a71fa4747c0a577")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Failed to run command 'some-nonexisting-program-b5d9574198b7e4b12a71fa4747c0a577'",
        ));
}

#[cfg(unix)]
#[test]
fn fails_for_failing_command_without_shell() {
    hyperfine()
        .arg("--shell=none")
        .arg("--runs=1")
        .arg("false")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Command terminated with non-zero exit code",
        ));
}

#[test]
fn fails_for_unknown_setup_command() {
    hyperfine()
        .arg("--runs=1")
        .arg("--setup=some-nonexisting-program-b5d9574198b7e4b12a71fa4747c0a577")
        .arg("echo test")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "The setup command terminated with a non-zero exit code.",
        ));
}

#[test]
fn fails_for_unknown_cleanup_command() {
    hyperfine()
        .arg("--runs=1")
        .arg("--cleanup=some-nonexisting-program-b5d9574198b7e4b12a71fa4747c0a577")
        .arg("echo test")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "The cleanup command terminated with a non-zero exit code.",
        ));
}

#[test]
fn fails_for_unknown_prepare_command() {
    hyperfine()
        .arg("--prepare=some-nonexisting-program-b5d9574198b7e4b12a71fa4747c0a577")
        .arg("echo test")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "The preparation command terminated with a non-zero exit code.",
        ));
}

#[test]
fn fails_for_unknown_conclude_command() {
    hyperfine()
        .arg("--conclude=some-nonexisting-program-b5d9574198b7e4b12a71fa4747c0a577")
        .arg("echo test")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "The conclusion command terminated with a non-zero exit code.",
        ));
}

#[cfg(unix)]
#[test]
fn can_run_failing_commands_with_ignore_failure_option() {
    hyperfine()
        .arg("false")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Command terminated with non-zero exit code",
        ));

    hyperfine()
        .arg("--runs=1")
        .arg("--ignore-failure")
        .arg("false")
        .assert()
        .success();
}

#[test]
fn shows_output_of_benchmarked_command() {
    hyperfine()
        .arg("--runs=2")
        .arg("--command-name=dummy")
        .arg("--show-output")
        .arg("echo 4fd47015")
        .assert()
        .success()
        .stdout(predicate::str::contains("4fd47015").count(2));
}

#[test]
fn runs_commands_using_user_defined_shell() {
    hyperfine()
        .arg("--runs=1")
        .arg("--show-output")
        .arg("--shell")
        .arg("echo 'custom_shell' '--shell-arg'")
        .arg("echo benchmark")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("custom_shell --shell-arg -c echo benchmark").or(
                predicate::str::contains("custom_shell --shell-arg /C echo benchmark"),
            ),
        );
}

#[test]
fn can_pass_input_to_command_from_a_file() {
    hyperfine()
        .arg("--runs=1")
        .arg("--input=example_input_file.txt")
        .arg("--show-output")
        .arg(STDIN_READ_COMMAND)
        .assert()
        .success()
        .stdout(predicate::str::contains("This text is part of a file"));
}

#[test]
fn fails_if_invalid_stdin_data_file_provided() {
    hyperfine()
        .arg("--runs=1")
        .arg("--input=example_non_existent_file.txt")
        .arg("--show-output")
        .arg(STDIN_READ_COMMAND)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "The file 'example_non_existent_file.txt' specified as '--input' does not exist",
        ));
}

#[test]
fn returns_mean_time_in_correct_unit() {
    hyperfine_debug()
        .arg("sleep 1.234")
        .assert()
        .success()
        .stdout(predicate::str::contains("Time (mean ± σ):      1.234 s ±"));

    hyperfine_debug()
        .arg("sleep 0.123")
        .assert()
        .success()
        .stdout(predicate::str::contains("Time (mean ± σ):     123.0 ms ±"));

    hyperfine_debug()
        .arg("--time-unit=millisecond")
        .arg("sleep 1.234")
        .assert()
        .success()
        .stdout(predicate::str::contains("Time (mean ± σ):     1234.0 ms ±"));

    hyperfine_debug()
        .arg("--time-unit=microsecond")
        .arg("sleep 1.234")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Time (mean ± σ):     1234000.0 µs ±",
        ));
}

#[test]
fn performs_ten_runs_for_slow_commands() {
    hyperfine_debug()
        .arg("sleep 0.5")
        .assert()
        .success()
        .stdout(predicate::str::contains("10 runs"));
}

#[test]
fn performs_three_seconds_of_benchmarking_for_fast_commands() {
    hyperfine_debug()
        .arg("sleep 0.01")
        .assert()
        .success()
        .stdout(predicate::str::contains("300 runs"));
}

#[test]
fn takes_shell_spawning_time_into_account_for_computing_number_of_runs() {
    hyperfine_debug()
        .arg("--shell=sleep 0.02")
        .arg("sleep 0.01")
        .assert()
        .success()
        .stdout(predicate::str::contains("100 runs"));
}

#[test]
fn takes_preparation_command_into_account_for_computing_number_of_runs() {
    hyperfine_debug()
        .arg("--prepare=sleep 0.02")
        .arg("sleep 0.01")
        .assert()
        .success()
        .stdout(predicate::str::contains("100 runs"));

    // Shell overhead needs to be added to both the prepare command and the actual command,
    // leading to a total benchmark time of (prepare + shell + cmd + shell = 0.1 s)
    hyperfine_debug()
        .arg("--shell=sleep 0.01")
        .arg("--prepare=sleep 0.03")
        .arg("sleep 0.05")
        .assert()
        .success()
        .stdout(predicate::str::contains("30 runs"));
}

#[test]
fn takes_conclusion_command_into_account_for_computing_number_of_runs() {
    hyperfine_debug()
        .arg("--conclude=sleep 0.02")
        .arg("sleep 0.01")
        .assert()
        .success()
        .stdout(predicate::str::contains("100 runs"));

    // Shell overhead needs to be added to both the conclude command and the actual command,
    // leading to a total benchmark time of (cmd + shell + conclude + shell = 0.1 s)
    hyperfine_debug()
        .arg("--shell=sleep 0.01")
        .arg("--conclude=sleep 0.03")
        .arg("sleep 0.05")
        .assert()
        .success()
        .stdout(predicate::str::contains("30 runs"));
}

#[test]
fn takes_both_preparation_and_conclusion_command_into_account_for_computing_number_of_runs() {
    hyperfine_debug()
        .arg("--prepare=sleep 0.01")
        .arg("--conclude=sleep 0.01")
        .arg("sleep 0.01")
        .assert()
        .success()
        .stdout(predicate::str::contains("100 runs"));

    // Shell overhead needs to be added to both the prepare, conclude and the actual command,
    // leading to a total benchmark time of (prepare + shell + cmd + shell + conclude + shell = 0.1 s)
    hyperfine_debug()
        .arg("--shell=sleep 0.01")
        .arg("--prepare=sleep 0.01")
        .arg("--conclude=sleep 0.01")
        .arg("sleep 0.05")
        .assert()
        .success()
        .stdout(predicate::str::contains("30 runs"));
}

#[test]
fn shows_benchmark_comparison_with_relative_times() {
    hyperfine_debug()
        .arg("sleep 1.0")
        .arg("sleep 2.0")
        .arg("sleep 3.0")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("2.00 ± 0.00 times faster")
                .and(predicate::str::contains("3.00 ± 0.00 times faster")),
        );
}

#[test]
fn shows_benchmark_comparison_with_same_time() {
    hyperfine_debug()
        .arg("--command-name=A")
        .arg("--command-name=B")
        .arg("sleep 1.0")
        .arg("sleep 1.0")
        .arg("sleep 2.0")
        .arg("sleep 1000.0")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("As fast (1.00 ± 0.00) as")
                .and(predicate::str::contains("2.00 ± 0.00 times faster"))
                .and(predicate::str::contains("1000.00 ± 0.00 times faster")),
        );
}

#[test]
fn shows_benchmark_comparison_relative_to_reference() {
    hyperfine_debug()
        .arg("--reference=sleep 2.0")
        .arg("sleep 1.0")
        .arg("sleep 3.0")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("2.00 ± 0.00 times slower")
                .and(predicate::str::contains("1.50 ± 0.00 times faster")),
        );
}

#[test]
fn performs_all_benchmarks_in_parameter_scan() {
    hyperfine_debug()
        .arg("--parameter-scan")
        .arg("time")
        .arg("30")
        .arg("45")
        .arg("--parameter-step-size")
        .arg("5")
        .arg("sleep {time}")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Benchmark 1: sleep 30")
                .and(predicate::str::contains("Benchmark 2: sleep 35"))
                .and(predicate::str::contains("Benchmark 3: sleep 40"))
                .and(predicate::str::contains("Benchmark 4: sleep 45"))
                .and(predicate::str::contains("Benchmark 5: sleep 50").not()),
        );
}

#[test]
fn performs_reference_and_all_benchmarks_in_parameter_scan() {
    hyperfine_debug()
        .arg("--reference=sleep 25")
        .arg("--parameter-scan")
        .arg("time")
        .arg("30")
        .arg("45")
        .arg("--parameter-step-size")
        .arg("5")
        .arg("sleep {time}")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Benchmark 1: sleep 25")
                .and(predicate::str::contains("Benchmark 2: sleep 30"))
                .and(predicate::str::contains("Benchmark 3: sleep 35"))
                .and(predicate::str::contains("Benchmark 4: sleep 40"))
                .and(predicate::str::contains("Benchmark 5: sleep 45"))
                .and(predicate::str::contains("Benchmark 6: sleep 50").not()),
        );
}

#[test]
fn intermediate_results_are_not_exported_to_stdout() {
    hyperfine_debug()
        .arg("--style=none") // To only see the Markdown export on stdout
        .arg("--export-markdown")
        .arg("-")
        .arg("sleep 1")
        .arg("sleep 2")
        .assert()
        .success()
        .stdout(
            (predicate::str::contains("sleep 1").count(1))
                .and(predicate::str::contains("sleep 2").count(1)),
        );
}

#[test]
#[cfg(unix)]
fn exports_intermediate_results_to_file() {
    use tempfile::tempdir;

    let tempdir = tempdir().unwrap();
    let export_path = tempdir.path().join("results.md");

    hyperfine()
        .arg("--runs=1")
        .arg("--export-markdown")
        .arg(&export_path)
        .arg("true")
        .arg("false")
        .assert()
        .failure();

    let contents = std::fs::read_to_string(export_path).unwrap();
    assert!(contents.contains("true"));
}

#[test]
fn unused_parameters_are_shown_in_benchmark_name() {
    hyperfine()
        .arg("--runs=2")
        .arg("--parameter-list")
        .arg("branch")
        .arg("master,feature")
        .arg("echo test")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("echo test (branch = master)")
                .and(predicate::str::contains("echo test (branch = feature)")),
        );
}

#[test]
fn speed_comparison_sort_order() {
    for sort_order in ["auto", "mean-time"] {
        hyperfine_debug()
            .arg("sleep 2")
            .arg("sleep 1")
            .arg(format!("--sort={sort_order}"))
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "sleep 1 ran\n    2.00 ± 0.00 times faster than sleep 2",
            ));
    }

    hyperfine_debug()
        .arg("sleep 2")
        .arg("sleep 1")
        .arg("--sort=command")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "2.00 ±  0.00  sleep 2\n        1.00          sleep 1",
        ));
}

#[cfg(windows)]
#[test]
fn windows_quote_args() {
    hyperfine()
        .arg("more \"example_input_file.txt\"")
        .assert()
        .success();
}

#[cfg(windows)]
#[test]
fn windows_quote_before_quote_args() {
    hyperfine()
        .arg("dir \"..\\src\\\" \"..\\tests\\\"")
        .assert()
        .success();
}
