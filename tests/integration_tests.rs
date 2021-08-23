use std::{fs::File, io::Read, path::PathBuf, process::Command};

use assert_cmd::cargo::CommandCargoExt;
use tempfile::{tempdir, TempDir};

fn hyperfine_raw_command() -> Command {
    let mut cmd = Command::cargo_bin("hyperfine").unwrap();
    cmd.current_dir("tests/");
    cmd
}

fn hyperfine() -> assert_cmd::Command {
    assert_cmd::Command::from_std(hyperfine_raw_command())
}

#[test]
fn hyperfine_runs_successfully() {
    hyperfine()
        .arg("--runs=2")
        .arg("echo dummy benchmark")
        .assert()
        .success();
}

#[test]
fn at_least_two_runs_are_required() {
    hyperfine()
        .arg("--runs=1")
        .arg("echo dummy benchmark")
        .assert()
        .failure();
}

struct ExecutionOrderTest {
    cmd: assert_cmd::Command,
    expected_content: String,
    logfile_path: PathBuf,
    #[allow(dead_code)]
    tempdir: TempDir,
}

impl ExecutionOrderTest {
    fn new() -> Self {
        let tempdir = tempdir().unwrap();
        let logfile_path = tempdir.path().join("output.log");

        ExecutionOrderTest {
            cmd: hyperfine(),
            expected_content: String::new(),
            logfile_path,
            tempdir,
        }
    }

    fn arg<S: AsRef<str>>(&mut self, arg: S) -> &mut Self {
        self.cmd.arg(arg.as_ref());
        self
    }

    fn get_command(&self, output: &str) -> String {
        format!(
            "echo {output} >> {path}",
            output = output,
            path = self.logfile_path.to_string_lossy()
        )
    }

    fn command(&mut self, output: &str) -> &mut Self {
        self.arg(self.get_command(output));
        self
    }

    fn prepare(&mut self, output: &str) -> &mut Self {
        self.arg("--prepare");
        self.command(output)
    }

    fn cleanup(&mut self, output: &str) -> &mut Self {
        self.arg("--cleanup");
        self.command(output)
    }

    fn expect_output(&mut self, output: &str) -> &mut Self {
        self.expected_content.push_str(output);

        #[cfg(windows)]
        {
            self.expected_content.push_str(" \r");
        }

        self.expected_content.push('\n');
        self
    }

    fn run(&mut self) {
        self.cmd.assert().success();

        let mut f = File::open(&self.logfile_path).unwrap();
        let mut content = String::new();
        f.read_to_string(&mut content).unwrap();

        assert_eq!(content, self.expected_content);
    }
}

impl Default for ExecutionOrderTest {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn benchmarks_are_executed_sequentially() {
    ExecutionOrderTest::new()
        .arg("--runs=2")
        .command("command 1")
        .command("command 2")
        .expect_output("command 1")
        .expect_output("command 1")
        .expect_output("command 2")
        .expect_output("command 2")
        .run();
}

#[test]
fn warmup_runs_are_executed_before_benchmarking_runs() {
    ExecutionOrderTest::new()
        .arg("--runs=2")
        .arg("--warmup=3")
        .command("command 1")
        .expect_output("command 1")
        .expect_output("command 1")
        .expect_output("command 1")
        .expect_output("command 1")
        .expect_output("command 1")
        .run();
}

#[test]
fn prepare_commands_are_executed_before_each_timing_run() {
    ExecutionOrderTest::new()
        .arg("--runs=2")
        .prepare("prepare")
        .command("command 1")
        .command("command 2")
        .expect_output("prepare")
        .expect_output("command 1")
        .expect_output("prepare")
        .expect_output("command 1")
        .expect_output("prepare")
        .expect_output("command 2")
        .expect_output("prepare")
        .expect_output("command 2")
        .run();
}

#[test]
fn cleanup_commands_are_executed_once_after_each_benchmark() {
    ExecutionOrderTest::new()
        .arg("--runs=2")
        .cleanup("cleanup")
        .command("command 1")
        .command("command 2")
        .expect_output("command 1")
        .expect_output("command 1")
        .expect_output("cleanup")
        .expect_output("command 2")
        .expect_output("command 2")
        .expect_output("cleanup")
        .run();
}

#[test]
fn single_parameter_value() {
    ExecutionOrderTest::new()
        .arg("--runs=2")
        .arg("--parameter-list")
        .arg("number")
        .arg("1,2,3")
        .command("command {number}")
        .expect_output("command 1")
        .expect_output("command 1")
        .expect_output("command 2")
        .expect_output("command 2")
        .expect_output("command 3")
        .expect_output("command 3")
        .run();
}

#[test]
fn multiple_parameter_values() {
    ExecutionOrderTest::new()
        .arg("--runs=2")
        .arg("--parameter-list")
        .arg("number")
        .arg("1,2,3")
        .arg("--parameter-list")
        .arg("letter")
        .arg("a,b")
        .command("command {number} {letter}")
        .expect_output("command 1 a")
        .expect_output("command 1 a")
        .expect_output("command 2 a")
        .expect_output("command 2 a")
        .expect_output("command 3 a")
        .expect_output("command 3 a")
        .expect_output("command 1 b")
        .expect_output("command 1 b")
        .expect_output("command 2 b")
        .expect_output("command 2 b")
        .expect_output("command 3 b")
        .expect_output("command 3 b")
        .run();
}
