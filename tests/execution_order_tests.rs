use std::{fs, path::PathBuf};

use tempfile::{tempdir, TempDir};

mod common;
use common::hyperfine;

struct ExecutionOrderTest {
    cmd: assert_cmd::Command,
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

    fn setup(&mut self, output: &str) -> &mut Self {
        self.arg("--setup");
        self.command(output)
    }

    fn prepare(&mut self, output: &str) -> &mut Self {
        self.arg("--prepare");
        self.command(output)
    }

    fn reference(&mut self, output: &str) -> &mut Self {
        self.arg("--reference");
        self.command(output)
    }

    fn conclude(&mut self, output: &str) -> &mut Self {
        self.arg("--conclude");
        self.command(output)
    }

    fn cleanup(&mut self, output: &str) -> &mut Self {
        self.arg("--cleanup");
        self.command(output)
    }

    fn run(&mut self) -> String {
        self.cmd.assert().success();

        let content = fs::read_to_string(&self.logfile_path).unwrap();
        // cmd.exe's echo includes the space before redirection and uses CRLF.
        #[cfg(windows)]
        let content = content.replace(" \r\n", "\n");
        content
    }
}

impl Default for ExecutionOrderTest {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn benchmarks_are_executed_sequentially_one() {
    let output = ExecutionOrderTest::new()
        .arg("--runs=1")
        .command("command 1")
        .command("command 2")
        .run();

    insta::assert_snapshot!(output, @r"
    command 1
    command 2
    ");
}

#[test]
fn benchmarks_are_executed_sequentially() {
    let output = ExecutionOrderTest::new()
        .arg("--runs=2")
        .command("command 1")
        .command("command 2")
        .run();

    insta::assert_snapshot!(output, @r"
    command 1
    command 1
    command 2
    command 2
    ");
}

#[test]
fn warmup_runs_are_executed_before_benchmarking_runs() {
    let output = ExecutionOrderTest::new()
        .arg("--runs=2")
        .arg("--warmup=3")
        .command("command 1")
        .run();

    insta::assert_snapshot!(output, @r"
    command 1
    command 1
    command 1
    command 1
    command 1
    ");
}

#[test]
fn setup_commands_are_executed_before_each_series_of_timing_runs() {
    let output = ExecutionOrderTest::new()
        .arg("--runs=2")
        .setup("setup")
        .command("command 1")
        .command("command 2")
        .run();

    insta::assert_snapshot!(output, @r"
    setup
    command 1
    command 1
    setup
    command 2
    command 2
    ");
}

#[test]
fn prepare_commands_are_executed_before_each_timing_run() {
    let output = ExecutionOrderTest::new()
        .arg("--runs=2")
        .prepare("prepare")
        .command("command 1")
        .command("command 2")
        .run();

    insta::assert_snapshot!(output, @r"
    prepare
    command 1
    prepare
    command 1
    prepare
    command 2
    prepare
    command 2
    ");
}

#[test]
fn conclude_commands_are_executed_after_each_timing_run() {
    let output = ExecutionOrderTest::new()
        .arg("--runs=2")
        .conclude("conclude")
        .command("command 1")
        .command("command 2")
        .run();

    insta::assert_snapshot!(output, @r"
    command 1
    conclude
    command 1
    conclude
    command 2
    conclude
    command 2
    conclude
    ");
}

#[test]
fn prepare_commands_are_executed_before_each_warmup() {
    let output = ExecutionOrderTest::new()
        .arg("--warmup=2")
        .arg("--runs=1")
        .prepare("prepare")
        .command("command 1")
        .command("command 2")
        .run();

    insta::assert_snapshot!(output, @r"
    prepare
    command 1
    prepare
    command 1
    prepare
    command 1
    prepare
    command 2
    prepare
    command 2
    prepare
    command 2
    ");
}

#[test]
fn conclude_commands_are_executed_after_each_warmup() {
    let output = ExecutionOrderTest::new()
        .arg("--warmup=2")
        .arg("--runs=1")
        .conclude("conclude")
        .command("command 1")
        .command("command 2")
        .run();

    insta::assert_snapshot!(output, @r"
    command 1
    conclude
    command 1
    conclude
    command 1
    conclude
    command 2
    conclude
    command 2
    conclude
    command 2
    conclude
    ");
}

#[test]
fn cleanup_commands_are_executed_once_after_each_benchmark() {
    let output = ExecutionOrderTest::new()
        .arg("--runs=2")
        .cleanup("cleanup")
        .command("command 1")
        .command("command 2")
        .run();

    insta::assert_snapshot!(output, @r"
    command 1
    command 1
    cleanup
    command 2
    command 2
    cleanup
    ");
}

#[test]
fn setup_prepare_cleanup_combined() {
    let output = ExecutionOrderTest::new()
        .arg("--warmup=1")
        .arg("--runs=2")
        .setup("setup")
        .prepare("prepare")
        .command("command1")
        .command("command2")
        .cleanup("cleanup")
        .run();

    insta::assert_snapshot!(output, @r"
    setup
    prepare
    command1
    prepare
    command1
    prepare
    command1
    cleanup
    setup
    prepare
    command2
    prepare
    command2
    prepare
    command2
    cleanup
    ");
}

#[test]
fn setup_prepare_conclude_cleanup_combined() {
    let output = ExecutionOrderTest::new()
        .arg("--warmup=1")
        .arg("--runs=2")
        .setup("setup")
        .prepare("prepare")
        .command("command1")
        .command("command2")
        .conclude("conclude")
        .cleanup("cleanup")
        .run();

    insta::assert_snapshot!(output, @r"
    setup
    prepare
    command1
    conclude
    prepare
    command1
    conclude
    prepare
    command1
    conclude
    cleanup
    setup
    prepare
    command2
    conclude
    prepare
    command2
    conclude
    prepare
    command2
    conclude
    cleanup
    ");
}

#[test]
fn single_parameter_value() {
    let output = ExecutionOrderTest::new()
        .arg("--runs=2")
        .arg("--parameter-list")
        .arg("number")
        .arg("1,2,3")
        .command("command {number}")
        .run();

    insta::assert_snapshot!(output, @r"
    command 1
    command 1
    command 2
    command 2
    command 3
    command 3
    ");
}

#[test]
fn multiple_parameter_values() {
    let output = ExecutionOrderTest::new()
        .arg("--runs=2")
        .arg("--parameter-list")
        .arg("number")
        .arg("1,2,3")
        .arg("--parameter-list")
        .arg("letter")
        .arg("a,b")
        .command("command {number} {letter}")
        .run();

    insta::assert_snapshot!(output, @r"
    command 1 a
    command 1 a
    command 2 a
    command 2 a
    command 3 a
    command 3 a
    command 1 b
    command 1 b
    command 2 b
    command 2 b
    command 3 b
    command 3 b
    ");
}

#[test]
fn reference_is_executed_first() {
    let output = ExecutionOrderTest::new()
        .arg("--runs=1")
        .reference("reference")
        .command("command 1")
        .command("command 2")
        .run();

    insta::assert_snapshot!(output, @r"
    reference
    command 1
    command 2
    ");
}

#[test]
fn reference_is_executed_first_parameter_value() {
    let output = ExecutionOrderTest::new()
        .arg("--runs=2")
        .reference("reference")
        .arg("--parameter-list")
        .arg("number")
        .arg("1,2,3")
        .command("command {number}")
        .run();

    insta::assert_snapshot!(output, @r"
    reference
    reference
    command 1
    command 1
    command 2
    command 2
    command 3
    command 3
    ");
}

#[test]
fn reference_is_executed_separately_from_commands() {
    let output = ExecutionOrderTest::new()
        .arg("--runs=1")
        .reference("command 1")
        .command("command 1")
        .command("command 2")
        .run();

    insta::assert_snapshot!(output, @r"
    command 1
    command 1
    command 2
    ");
}

#[test]
fn setup_prepare_reference_conclude_cleanup_combined() {
    let output = ExecutionOrderTest::new()
        .arg("--warmup=1")
        .arg("--runs=2")
        .setup("setup")
        .prepare("prepare")
        .reference("reference")
        .command("command1")
        .command("command2")
        .conclude("conclude")
        .cleanup("cleanup")
        .run();

    insta::assert_snapshot!(output, @r"
    setup
    prepare
    reference
    conclude
    prepare
    reference
    conclude
    prepare
    reference
    conclude
    cleanup
    setup
    prepare
    command1
    conclude
    prepare
    command1
    conclude
    prepare
    command1
    conclude
    cleanup
    setup
    prepare
    command2
    conclude
    prepare
    command2
    conclude
    prepare
    command2
    conclude
    cleanup
    ");
}

#[test]
fn setup_separate_prepare_separate_conclude_cleanup_combined() {
    let output = ExecutionOrderTest::new()
        .arg("--warmup=1")
        .arg("--runs=2")
        .setup("setup")
        .cleanup("cleanup")
        .prepare("prepare1")
        .command("command1")
        .conclude("conclude1")
        .prepare("prepare2")
        .command("command2")
        .conclude("conclude2")
        .run();

    insta::assert_snapshot!(output, @r"
    setup
    prepare1
    command1
    conclude1
    prepare1
    command1
    conclude1
    prepare1
    command1
    conclude1
    cleanup
    setup
    prepare2
    command2
    conclude2
    prepare2
    command2
    conclude2
    prepare2
    command2
    conclude2
    cleanup
    ");
}

#[test]
fn setup_separate_prepare_reference_separate_conclude_cleanup_combined() {
    let output = ExecutionOrderTest::new()
        .arg("--warmup=1")
        .arg("--runs=2")
        .setup("setup")
        .cleanup("cleanup")
        .prepare("prepareref")
        .reference("reference")
        .conclude("concluderef")
        .prepare("prepare1")
        .command("command1")
        .conclude("conclude1")
        .prepare("prepare2")
        .command("command2")
        .conclude("conclude2")
        .run();

    insta::assert_snapshot!(output, @r"
    setup
    prepareref
    reference
    concluderef
    prepareref
    reference
    concluderef
    prepareref
    reference
    concluderef
    cleanup
    setup
    prepare1
    command1
    conclude1
    prepare1
    command1
    conclude1
    prepare1
    command1
    conclude1
    cleanup
    setup
    prepare2
    command2
    conclude2
    prepare2
    command2
    conclude2
    prepare2
    command2
    conclude2
    cleanup
    ");
}
