mod common;
use common::{hyperfine, hyperfine_raw_command};

use assert_cmd::assert::OutputAssertExt;
use insta_cmd::assert_cmd_snapshot;
use predicates::prelude::*;

/// Platform-specific I/O utility.
/// - On Unix-like systems, defaults to `cat`.
/// - On Windows, uses `findstr` as an alternative.
///   See: <https://superuser.com/questions/853580/real-windows-equivalent-to-cat-stdin>
const STDIN_READ_COMMAND: &str = if cfg!(windows) { "findstr x*" } else { "cat" };

pub fn hyperfine_debug() -> std::process::Command {
    let mut cmd = hyperfine_raw_command();
    cmd.arg("--debug-mode");
    cmd
}

fn snapshot_settings() -> insta::Settings {
    let mut settings = insta::Settings::clone_current();
    // Ignore trailing report padding while preserving indentation and internal spacing.
    settings.add_filter(r"(?m)[ \t]+$", "");
    settings
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
fn generates_shell_completion() {
    hyperfine()
        .arg("generate-shell-completion")
        .arg("fish")
        .assert()
        .success()
        .stdout(predicate::str::contains("complete").and(predicate::str::contains("hyperfine")));
}

#[test]
fn rejects_unknown_shell_completion() {
    hyperfine()
        .arg("generate-shell-completion")
        .arg("unknown")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'unknown'"));
}

#[test]
fn rejects_benchmark_command_with_shell_completion_subcommand() {
    hyperfine()
        .arg("echo ok")
        .arg("generate-shell-completion")
        .arg("fish")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn one_run_is_supported() {
    hyperfine()
        .arg("--runs=1")
        .arg("echo dummy benchmark")
        .assert()
        .success();
}

/// Regression test: hyperfine must not panic when writing to a closed
/// stdout pipe (e.g. `hyperfine ... | head -n 1` or a downstream process
/// that terminates early). A `BrokenPipe` error should result in a quiet
/// exit with code 0 instead of a `println!` panic.
#[test]
fn exits_quietly_when_stdout_is_closed() {
    use std::process::Stdio;

    for args in [
        vec!["--runs=1", "echo dummy benchmark"],
        vec!["--runs=1", "--export-json=-", "echo dummy benchmark"],
        vec!["--runs=1", "echo dummy benchmark", "echo second benchmark"],
    ] {
        // A pipe whose read end has been closed before hyperfine is even
        // started: every write to it will fail with `BrokenPipe`.
        let (reader, writer) = std::io::pipe().expect("failed to create pipe");
        drop(reader);

        let output = hyperfine_raw_command()
            .args(&args)
            .stdout(writer)
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to spawn hyperfine")
            .wait_with_output()
            .expect("failed to wait for hyperfine");
        let stderr = String::from_utf8_lossy(&output.stderr);

        assert!(
            !stderr.contains("panicked"),
            "hyperfine panicked while writing to a closed pipe:\n{}",
            stderr
        );
        assert!(
            output.status.success(),
            "hyperfine did not exit cleanly; stderr:\n{}",
            stderr
        );
    }
}

#[test]
fn fails_with_zero_runs() {
    for option in ["--runs", "--min-runs", "--max-runs"] {
        hyperfine()
            .arg(option)
            .arg("0")
            .arg("echo dummy benchmark")
            .assert()
            .code(2)
            .stderr(predicate::str::contains(format!(
                "invalid value '0' for '{option} <NUM>'"
            )));
    }
}

#[test]
fn min_runs_of_one_still_performs_one_run() {
    hyperfine_debug()
        .arg("--min-runs=1")
        .arg("--warmup=0")
        .arg("sleep 4")
        .assert()
        .success()
        .stdout(predicate::str::contains("Time (abs ≡)"));
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

#[cfg(unix)]
#[test]
fn can_ignore_specific_exit_codes() {
    // Test that specifying exit code 1 ignores it
    hyperfine()
        .arg("--runs=1")
        .arg("--ignore-failure=1")
        .arg("exit 1")
        .assert()
        .success();

    // Test that other exit codes still fail
    hyperfine()
        .arg("--runs=1")
        .arg("--ignore-failure=1")
        .arg("exit 2")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Command terminated with non-zero exit code 2",
        ));
}

#[cfg(unix)]
#[test]
fn can_ignore_multiple_exit_codes() {
    // Test that all specified exit codes are ignored
    hyperfine()
        .arg("--runs=1")
        .arg("--ignore-failure=1,2,3")
        .arg("exit 1")
        .assert()
        .success();

    hyperfine()
        .arg("--runs=1")
        .arg("--ignore-failure=1,2,3")
        .arg("exit 2")
        .assert()
        .success();

    hyperfine()
        .arg("--runs=1")
        .arg("--ignore-failure=1,2,3")
        .arg("exit 3")
        .assert()
        .success();

    // Test that other exit codes still fail
    hyperfine()
        .arg("--runs=1")
        .arg("--ignore-failure=1,2,3")
        .arg("exit 4")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Command terminated with non-zero exit code 4",
        ));
}

#[cfg(unix)]
#[test]
fn ignore_failure_with_all_non_zero() {
    // Test explicit "all-non-zero" mode
    hyperfine()
        .arg("--runs=1")
        .arg("--ignore-failure=all-non-zero")
        .arg("exit 5")
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
    let _settings = snapshot_settings().bind_to_scope();
    assert_cmd_snapshot!(hyperfine_debug()
        .arg("--style=basic")
        .arg("sleep 1.0")
        .arg("sleep 2.0")
        .arg("sleep 3.0"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Benchmark 1: sleep 1.0
      Time (mean ± σ):      1.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):    1.000 s …  1.000 s    10 runs

    Benchmark 2: sleep 2.0
      Time (mean ± σ):      2.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):    2.000 s …  2.000 s    10 runs

    Benchmark 3: sleep 3.0
      Time (mean ± σ):      3.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):    3.000 s …  3.000 s    10 runs

    Summary
      sleep 1.0 ran
        2.00 ± 0.00 times faster than sleep 2.0
        3.00 ± 0.00 times faster than sleep 3.0

    ----- stderr -----
    ");
}

#[test]
fn shows_benchmark_comparison_with_same_time() {
    let _settings = snapshot_settings().bind_to_scope();
    assert_cmd_snapshot!(hyperfine_debug()
        .arg("--style=basic")
        .arg("--command-name=A")
        .arg("--command-name=B")
        .arg("sleep 1.0")
        .arg("sleep 1.0")
        .arg("sleep 2.0")
        .arg("sleep 1000.0"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Benchmark 1: A
      Time (mean ± σ):      1.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):    1.000 s …  1.000 s    10 runs

    Benchmark 2: B
      Time (mean ± σ):      1.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):    1.000 s …  1.000 s    10 runs

    Benchmark 3: sleep 2.0
      Time (mean ± σ):      2.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):    2.000 s …  2.000 s    10 runs

    Benchmark 4: sleep 1000.0
      Time (mean ± σ):     1000.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):   1000.000 s … 1000.000 s    10 runs

    Summary
      A ran
        As fast (1.00 ± 0.00) as B
        2.00 ± 0.00 times faster than sleep 2.0
     1000.00 ± 0.00 times faster than sleep 1000.0

    ----- stderr -----
    ");
}

#[test]
fn shows_benchmark_comparison_relative_to_reference() {
    let _settings = snapshot_settings().bind_to_scope();
    assert_cmd_snapshot!(hyperfine_debug()
        .arg("--style=basic")
        .arg("--reference=sleep 2.0")
        .arg("sleep 1.0")
        .arg("sleep 3.0"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Benchmark 1: sleep 2.0
      Time (mean ± σ):      2.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):    2.000 s …  2.000 s    10 runs

    Benchmark 2: sleep 1.0
      Time (mean ± σ):      1.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):    1.000 s …  1.000 s    10 runs

    Benchmark 3: sleep 3.0
      Time (mean ± σ):      3.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):    3.000 s …  3.000 s    10 runs

    Summary
      sleep 2.0 ran
        2.00 ± 0.00 times slower than sleep 1.0
        1.50 ± 0.00 times faster than sleep 3.0

    ----- stderr -----
    ");
}

#[test]
fn shows_reference_name() {
    let _settings = snapshot_settings().bind_to_scope();
    assert_cmd_snapshot!(hyperfine_debug()
        .arg("--style=basic")
        .arg("--reference=sleep 2.0")
        .arg("--reference-name=refabc123")
        .arg("sleep 1.0")
        .arg("sleep 3.0"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Benchmark 1: refabc123
      Time (mean ± σ):      2.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):    2.000 s …  2.000 s    10 runs

    Benchmark 2: sleep 1.0
      Time (mean ± σ):      1.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):    1.000 s …  1.000 s    10 runs

    Benchmark 3: sleep 3.0
      Time (mean ± σ):      3.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):    3.000 s …  3.000 s    10 runs

    Summary
      refabc123 ran
        2.00 ± 0.00 times slower than sleep 1.0
        1.50 ± 0.00 times faster than sleep 3.0

    ----- stderr -----
    ");
}

#[test]
fn performs_all_benchmarks_in_parameter_scan() {
    let _settings = snapshot_settings().bind_to_scope();
    assert_cmd_snapshot!(hyperfine_debug()
        .arg("--style=basic")
        .arg("--parameter-scan")
        .arg("time")
        .arg("30")
        .arg("45")
        .arg("--parameter-step-size")
        .arg("5")
        .arg("sleep {time}"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Benchmark 1: sleep 30
      Time (mean ± σ):     30.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):   30.000 s … 30.000 s    10 runs

    Benchmark 2: sleep 35
      Time (mean ± σ):     35.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):   35.000 s … 35.000 s    10 runs

    Benchmark 3: sleep 40
      Time (mean ± σ):     40.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):   40.000 s … 40.000 s    10 runs

    Benchmark 4: sleep 45
      Time (mean ± σ):     45.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):   45.000 s … 45.000 s    10 runs

    Summary
      sleep 30 ran
        1.17 ± 0.00 times faster than sleep 35
        1.33 ± 0.00 times faster than sleep 40
        1.50 ± 0.00 times faster than sleep 45

    ----- stderr -----
    ");
}

#[test]
fn performs_reference_and_all_benchmarks_in_parameter_scan() {
    let _settings = snapshot_settings().bind_to_scope();
    assert_cmd_snapshot!(hyperfine_debug()
        .arg("--style=basic")
        .arg("--reference=sleep 25")
        .arg("--parameter-scan")
        .arg("time")
        .arg("30")
        .arg("45")
        .arg("--parameter-step-size")
        .arg("5")
        .arg("sleep {time}"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Benchmark 1: sleep 25
      Time (mean ± σ):     25.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):   25.000 s … 25.000 s    10 runs

    Benchmark 2: sleep 30
      Time (mean ± σ):     30.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):   30.000 s … 30.000 s    10 runs

    Benchmark 3: sleep 35
      Time (mean ± σ):     35.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):   35.000 s … 35.000 s    10 runs

    Benchmark 4: sleep 40
      Time (mean ± σ):     40.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):   40.000 s … 40.000 s    10 runs

    Benchmark 5: sleep 45
      Time (mean ± σ):     45.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):   45.000 s … 45.000 s    10 runs

    Summary
      sleep 25 ran
        1.20 ± 0.00 times faster than sleep 30
        1.40 ± 0.00 times faster than sleep 35
        1.60 ± 0.00 times faster than sleep 40
        1.80 ± 0.00 times faster than sleep 45

    ----- stderr -----
    ");
}

#[test]
fn intermediate_results_are_not_exported_to_stdout() {
    let _settings = snapshot_settings().bind_to_scope();
    assert_cmd_snapshot!(hyperfine_debug()
        .arg("--style=none") // To only see the Markdown export on stdout
        .arg("--export-markdown")
        .arg("-")
        .arg("sleep 1")
        .arg("sleep 2"), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    | Command | Mean [s] | Min [s] | Max [s] | Relative |
    |:---|---:|---:|---:|---:|
    | `sleep 1` | 1.000 ± 0.000 | 1.000 | 1.000 | 1.00 |
    | `sleep 2` | 2.000 ± 0.000 | 2.000 | 2.000 | 2.00 ± 0.00 |


    ----- stderr -----
    ");
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
    let _settings = snapshot_settings().bind_to_scope();
    insta::allow_duplicates! {
        for sort_order in ["auto", "mean-time"] {
            assert_cmd_snapshot!(hyperfine_debug()
                .arg("--style=basic")
                .arg("sleep 2")
                .arg("sleep 1")
                .arg(format!("--sort={sort_order}")), @r"
            success: true
            exit_code: 0
            ----- stdout -----
            Benchmark 1: sleep 2
              Time (mean ± σ):      2.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
              Range (min … max):    2.000 s …  2.000 s    10 runs

            Benchmark 2: sleep 1
              Time (mean ± σ):      1.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
              Range (min … max):    1.000 s …  1.000 s    10 runs

            Summary
              sleep 1 ran
                2.00 ± 0.00 times faster than sleep 2

            ----- stderr -----
            ");
        }
    }

    assert_cmd_snapshot!(hyperfine_debug()
        .arg("--style=basic")
        .arg("sleep 2")
        .arg("sleep 1")
        .arg("--sort=command"), @r"
    success: true
    exit_code: 0
    ----- stdout -----
    Benchmark 1: sleep 2
      Time (mean ± σ):      2.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):    2.000 s …  2.000 s    10 runs

    Benchmark 2: sleep 1
      Time (mean ± σ):      1.000 s ±  0.000 s    [User: 0.000 s, System: 0.000 s]
      Range (min … max):    1.000 s …  1.000 s    10 runs

    Relative speed comparison
            2.00 ±  0.00  sleep 2
            1.00          sleep 1

    ----- stderr -----
    ");
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
