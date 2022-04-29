mod common;
use common::hyperfine;

use predicates::prelude::*;

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
fn fails_with_wrong_number_of_setup_options() {
    hyperfine()
        .arg("--runs=1")
        .arg("--setup=echo a")
        .arg("--setup=echo b")
        .arg("echo a")
        .arg("echo b")
        .assert()
        .success();

    hyperfine()
        .arg("--runs=1")
        .arg("--setup=echo a")
        .arg("--setup=echo b")
        .arg("echo a")
        .arg("echo b")
        .arg("echo c")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "The '--setup' option has to be provided",
        ));
}
