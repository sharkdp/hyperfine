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

fn json_snapshot_settings() -> insta::Settings {
    let mut settings = snapshot_settings();
    for field in ["hyperfine_version", "start_time", "os", "architecture"] {
        settings.add_filter(
            &format!(r#"("{field}": ")[^"]+(")"#),
            format!(r#"${{1}}[{field}]$2"#),
        );
    }
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
        .arg("--shell=default")
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
        .arg("--shell=default")
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
        .arg("--shell=default")
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
        .arg("--shell=default")
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
        .arg("--shell=default")
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
        .arg("--shell=default")
        .arg("--runs=1")
        .arg("--ignore-failure=1")
        .arg("exit 1")
        .assert()
        .success();

    // Test that other exit codes still fail
    hyperfine()
        .arg("--shell=default")
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
        .arg("--shell=default")
        .arg("--runs=1")
        .arg("--ignore-failure=1,2,3")
        .arg("exit 1")
        .assert()
        .success();

    hyperfine()
        .arg("--shell=default")
        .arg("--runs=1")
        .arg("--ignore-failure=1,2,3")
        .arg("exit 2")
        .assert()
        .success();

    hyperfine()
        .arg("--shell=default")
        .arg("--runs=1")
        .arg("--ignore-failure=1,2,3")
        .arg("exit 3")
        .assert()
        .success();

    // Test that other exit codes still fail
    hyperfine()
        .arg("--shell=default")
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
        .arg("--shell=default")
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
fn command_sorted_comparison_and_markup_identify_reference() {
    let _settings = snapshot_settings().bind_to_scope();
    let directory = tempfile::tempdir().unwrap();
    let export_path = directory.path().join("results.md");
    let output = hyperfine_debug()
        .args([
            "--style=basic",
            "--runs=1",
            "--sort=command",
            "--reference",
            "sleep 2",
            "--reference-name",
            "baseline",
            "--export-markdown",
        ])
        .arg(&export_path)
        .args(["sleep 1", "sleep 2", "sleep 3"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let comparison = stdout.split_once("Relative speed comparison\n").unwrap().1;
    insta::assert_snapshot!(comparison, @r"
            1.00          baseline (reference)
            2.00          sleep 1 (faster)
            1.00          sleep 2 (same speed)
            1.50          sleep 3 (slower)
    ");
    let markdown = std::fs::read_to_string(export_path).unwrap();
    insta::assert_snapshot!(markdown, @r"
    | Command | Mean [s] | Min [s] | Max [s] | Relative |
    |:---|---:|---:|---:|---:|
    | `baseline` | 2.000 | 2.000 | 2.000 | 1.00 (reference) |
    | `sleep 1` | 1.000 | 1.000 | 1.000 | 2.00 (faster) |
    | `sleep 2` | 2.000 | 2.000 | 2.000 | 1.00 (same speed) |
    | `sleep 3` | 3.000 | 3.000 | 3.000 | 1.50 (slower) |
    ");

    // Equal zero times have no meaningful relative factor.
    let output = hyperfine_debug()
        .args([
            "--style=none",
            "--runs=1",
            "--export-markdown=-",
            "--reference",
            "sleep 0",
            "--reference-name",
            "baseline",
            "sleep 0",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let markdown = String::from_utf8(output.stdout).unwrap();
    let row = markdown
        .lines()
        .find(|line| line.contains("sleep 0"))
        .unwrap();
    assert!(row.ends_with("| N/A |"), "{}", row);
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
fn rejects_negative_parameter_steps() {
    for step in ["-1", "-1.0"] {
        hyperfine()
            .args(["--parameter-scan", "n", "0", "1"])
            .arg(format!("--parameter-step-size={step}"))
            .arg("echo {n}")
            .assert()
            .code(1)
            .stdout(predicate::str::is_empty())
            .stderr(predicate::str::contains(
                "Parameter step size must be positive",
            ));
    }
}

#[cfg(unix)]
#[test]
fn selects_reference_with_parameterized_prepare() {
    let directory = tempfile::tempdir().unwrap();
    let csv_path = directory.path().join("results.csv");
    let json_path = directory.path().join("results.json");
    let output = hyperfine_raw_command()
        .args([
            "--style=basic",
            "--shell=none",
            "--runs=1",
            "-L",
            "delay",
            "0.2,0.4,0.6",
            "--prepare",
            "sleep {delay}",
            "--reference",
            "echo a (delay = 0.4)",
            "--export-csv",
        ])
        .arg(&csv_path)
        .arg("--export-json")
        .arg(&json_path)
        .arg("echo a")
        .output()
        .unwrap();
    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("Benchmark ").count(), 3);
    for (number, delay) in ["0.2", "0.4", "0.6"].iter().enumerate() {
        assert!(
            stdout.contains(&format!(
                "Benchmark {}: echo a (delay = {})",
                number + 1,
                delay
            )),
            "{}",
            stdout
        );
    }
    assert!(stdout.contains("echo a (delay = 0.4) ran"), "{}", stdout);

    let mut csv = csv::Reader::from_path(csv_path).unwrap();
    assert!(csv
        .headers()
        .unwrap()
        .iter()
        .any(|header| header == "parameter_delay"));
    assert_eq!(
        csv.records().collect::<Result<Vec<_>, _>>().unwrap().len(),
        3
    );
    let json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(json_path).unwrap()).unwrap();
    let results = json["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(
        results
            .iter()
            .map(|result| result["parameters"]["delay"]["value"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["0.2", "0.4", "0.6"]
    );
}

#[test]
fn rejects_unmatched_reference_before_export_or_setup() {
    let directory = tempfile::tempdir().unwrap();
    let export_path = directory.path().join("results.csv");
    let marker_path = directory.path().join("setup-ran");
    std::fs::write(&export_path, "previous contents").unwrap();

    hyperfine()
        .args(["--reference", "sleep 1", "-P", "secs", "2", "3", "--setup"])
        .arg(format!("echo touched > \"{}\"", marker_path.display()))
        .arg("--export-csv")
        .arg(&export_path)
        .arg("sleep {secs}")
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains(
            "does not match any parameterized benchmark",
        ));
    assert_eq!(
        std::fs::read_to_string(export_path).unwrap(),
        "previous contents"
    );
    assert!(!marker_path.exists());
}

#[test]
fn rejects_ambiguous_parameterized_reference() {
    hyperfine_debug()
        .args([
            "-L",
            "x",
            "1,2",
            "--command-name",
            "case",
            "--reference",
            "case",
            "sleep {x}",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "matches multiple parameterized benchmarks",
        ));
}

#[cfg(unix)]
#[test]
fn intermediate_markdown_waits_for_selected_reference() {
    let directory = tempfile::tempdir().unwrap();
    let export_path = directory.path().join("results.md");
    hyperfine()
        .args([
            "--style=none",
            "--shell=none",
            "--runs=1",
            "-L",
            "delay",
            "0.01,0.02,0.03",
            "--reference",
            "sleep 0.02",
            "--prepare",
            "true",
            "--prepare",
            "false",
            "--prepare",
            "true",
            "--export-markdown",
        ])
        .arg(&export_path)
        .arg("sleep {delay}")
        .assert()
        .failure();
    let markdown = std::fs::read_to_string(export_path).unwrap();
    let row = markdown
        .lines()
        .find(|line| line.contains("sleep 0.01"))
        .unwrap();
    assert!(row.ends_with("| N/A |"), "{}", row);
}

#[test]
fn markdown_export_uses_selected_parameterized_reference() {
    let _settings = snapshot_settings().bind_to_scope();
    assert_cmd_snapshot!(hyperfine_debug()
        .args([
            "--style=none",
            "--runs=1",
            "--export-markdown=-",
            "-P",
            "x",
            "1",
            "3",
            "--command-name",
            "case-{x}",
            "--reference",
            "case-2",
            "sleep {x}",
        ]), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    | Command | Mean [s] | Min [s] | Max [s] | Relative |
    |:---|---:|---:|---:|---:|
    | `case-1` | 1.000 | 1.000 | 1.000 | 2.00 (faster) |
    | `case-2` | 2.000 | 2.000 | 2.000 | 1.00 (reference) |
    | `case-3` | 3.000 | 3.000 | 3.000 | 1.50 (slower) |


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
fn invalid_command_options_preserve_export_files() {
    use tempfile::tempdir;

    for (option, values) in [
        ("--prepare", ["echo first", "echo second"]),
        ("--conclude", ["echo first", "echo second"]),
        ("--output", ["null", "pipe"]),
    ] {
        let directory = tempdir().unwrap();
        let existing_export = directory.path().join("previous.json");
        let new_export = directory.path().join("new.csv");
        let previous_results =
            b"{\"results\":[{\"command\":\"previous benchmark\",\"mean\":1.0}]}\n";
        std::fs::write(&existing_export, previous_results).unwrap();

        hyperfine()
            .arg("--runs=1")
            .arg("--export-json")
            .arg(&existing_export)
            .arg("--export-csv")
            .arg(&new_export)
            .args([option, values[0], option, values[1]])
            .arg("echo benchmark")
            .assert()
            .failure()
            .stderr(predicate::str::contains(format!(
                "The '{option}' option has to be provided just once or N times"
            )));

        assert_eq!(
            std::fs::read(&existing_export).unwrap(),
            previous_results,
            "{option} validation must preserve previous results"
        );
        assert!(
            !new_export.exists(),
            "{} validation must not create export files",
            option
        );
    }
}

#[test]
fn markdown_export_preserves_backticks_and_pipes_in_command_names() {
    let _settings = snapshot_settings().bind_to_scope();
    assert_cmd_snapshot!(hyperfine_debug()
        .arg("--style=none")
        .arg("--export-markdown=-")
        .arg("--command-name=echo `uname` | cat")
        .arg("sleep 1"), @r"
    success: true
    exit_code: 0
    ----- stdout -----

    | Command | Mean [s] | Min [s] | Max [s] | Relative |
    |:---|---:|---:|---:|---:|
    | `` echo `uname` \| cat `` | 1.000 ± 0.000 | 1.000 | 1.000 | 1.00 |


    ----- stderr -----
    ");
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
        .arg("--shell=default")
        .arg("more \"example_input_file.txt\"")
        .assert()
        .success();
}

#[cfg(windows)]
#[test]
fn windows_quote_before_quote_args() {
    hyperfine()
        .arg("--shell=default")
        .arg("dir \"..\\src\\\" \"..\\tests\\\"")
        .assert()
        .success();
}

#[test]
fn hyperfine_iteration_env_var_in_prepare_and_conclude_commands() {
    let tempdir = tempfile::tempdir().unwrap();
    let output_path = tempdir.path().join("iteration output.txt");
    let iteration = if cfg!(windows) {
        "%HYPERFINE_ITERATION%"
    } else {
        "$HYPERFINE_ITERATION"
    };
    let command = |phase| format!(r#"echo {phase}:{iteration} >> "iteration output.txt""#);

    hyperfine()
        .arg("--shell=default")
        .current_dir(tempdir.path())
        .arg("--runs=2")
        .arg("--warmup=1")
        .arg("--prepare")
        .arg(command("prepare"))
        .arg(command("benchmark"))
        .arg("--conclude")
        .arg(command("conclude"))
        .assert()
        .success();

    let contents = std::fs::read_to_string(output_path).unwrap();
    // cmd.exe's echo includes the space before the redirection operator.
    let lines: Vec<&str> = contents.lines().map(str::trim_end).collect();
    assert_eq!(
        lines,
        [
            "prepare:warmup-0",
            "benchmark:warmup-0",
            "conclude:warmup-0",
            "prepare:0",
            "benchmark:0",
            "conclude:0",
            "prepare:1",
            "benchmark:1",
            "conclude:1",
        ]
    );
}

#[test]
fn json_export_basic() {
    let _settings = json_snapshot_settings().bind_to_scope();
    assert_cmd_snapshot!(hyperfine_debug().args([
        "--style=none",
        "--export-json=-",
        "--runs=2",
        "--warmup=1",
        "--reference-name=one second",
        "--reference=sleep 1",
        "--command-name=sleep 2",
        "sleep 2",
    ]));
}

#[test]
fn json_export_single_run() {
    // Make sure that the standard deviation is set to `null`
    let _settings = json_snapshot_settings().bind_to_scope();
    assert_cmd_snapshot!(hyperfine_debug().args([
        "--style=none",
        "--export-json=-",
        "--runs=1",
        "sleep 1",
    ]));
}

#[test]
fn json_export_parameterized_with_reference() {
    let _settings = json_snapshot_settings().bind_to_scope();
    assert_cmd_snapshot!(hyperfine_debug().args([
        "--style=none",
        "--export-json=-",
        "--runs=2",
        "-L",
        "duration",
        "1,2",
        "--command-name=sleep for {duration} seconds",
        "--reference=sleep for 2 seconds",
        "sleep {duration}",
    ]));
}
