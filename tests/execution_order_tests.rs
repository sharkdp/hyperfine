use std::{fs::File, io::Read, path::PathBuf};

use tempfile::{tempdir, TempDir};

mod common;
use common::hyperfine;

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
fn benchmarks_are_executed_sequentially_one() {
    ExecutionOrderTest::new()
        .arg("--runs=1")
        .command("command 1")
        .command("command 2")
        .expect_output("command 1")
        .expect_output("command 2")
        .run();
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
fn setup_commands_are_executed_before_each_series_of_timing_runs() {
    ExecutionOrderTest::new()
        .arg("--runs=2")
        .setup("setup")
        .command("command 1")
        .command("command 2")
        .expect_output("setup")
        .expect_output("command 1")
        .expect_output("command 1")
        .expect_output("setup")
        .expect_output("command 2")
        .expect_output("command 2")
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
fn conclude_commands_are_executed_after_each_timing_run() {
    ExecutionOrderTest::new()
        .arg("--runs=2")
        .conclude("conclude")
        .command("command 1")
        .command("command 2")
        .expect_output("command 1")
        .expect_output("conclude")
        .expect_output("command 1")
        .expect_output("conclude")
        .expect_output("command 2")
        .expect_output("conclude")
        .expect_output("command 2")
        .expect_output("conclude")
        .run();
}

#[test]
fn prepare_commands_are_executed_before_each_warmup() {
    ExecutionOrderTest::new()
        .arg("--warmup=2")
        .arg("--runs=1")
        .prepare("prepare")
        .command("command 1")
        .command("command 2")
        // warmup 1
        .expect_output("prepare")
        .expect_output("command 1")
        .expect_output("prepare")
        .expect_output("command 1")
        // benchmark 1
        .expect_output("prepare")
        .expect_output("command 1")
        // warmup 2
        .expect_output("prepare")
        .expect_output("command 2")
        .expect_output("prepare")
        .expect_output("command 2")
        // benchmark 2
        .expect_output("prepare")
        .expect_output("command 2")
        .run();
}

#[test]
fn conclude_commands_are_executed_after_each_warmup() {
    ExecutionOrderTest::new()
        .arg("--warmup=2")
        .arg("--runs=1")
        .conclude("conclude")
        .command("command 1")
        .command("command 2")
        // warmup 1
        .expect_output("command 1")
        .expect_output("conclude")
        .expect_output("command 1")
        .expect_output("conclude")
        // benchmark 1
        .expect_output("command 1")
        .expect_output("conclude")
        // warmup 2
        .expect_output("command 2")
        .expect_output("conclude")
        .expect_output("command 2")
        .expect_output("conclude")
        // benchmark 2
        .expect_output("command 2")
        .expect_output("conclude")
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
fn setup_prepare_cleanup_combined() {
    ExecutionOrderTest::new()
        .arg("--warmup=1")
        .arg("--runs=2")
        .setup("setup")
        .prepare("prepare")
        .command("command1")
        .command("command2")
        .cleanup("cleanup")
        // 1
        .expect_output("setup")
        .expect_output("prepare")
        .expect_output("command1")
        .expect_output("prepare")
        .expect_output("command1")
        .expect_output("prepare")
        .expect_output("command1")
        .expect_output("cleanup")
        // 2
        .expect_output("setup")
        .expect_output("prepare")
        .expect_output("command2")
        .expect_output("prepare")
        .expect_output("command2")
        .expect_output("prepare")
        .expect_output("command2")
        .expect_output("cleanup")
        .run();
}

#[test]
fn setup_prepare_conclude_cleanup_combined() {
    ExecutionOrderTest::new()
        .arg("--warmup=1")
        .arg("--runs=2")
        .setup("setup")
        .prepare("prepare")
        .command("command1")
        .command("command2")
        .conclude("conclude")
        .cleanup("cleanup")
        // 1
        .expect_output("setup")
        .expect_output("prepare")
        .expect_output("command1")
        .expect_output("conclude")
        .expect_output("prepare")
        .expect_output("command1")
        .expect_output("conclude")
        .expect_output("prepare")
        .expect_output("command1")
        .expect_output("conclude")
        .expect_output("cleanup")
        // 2
        .expect_output("setup")
        .expect_output("prepare")
        .expect_output("command2")
        .expect_output("conclude")
        .expect_output("prepare")
        .expect_output("command2")
        .expect_output("conclude")
        .expect_output("prepare")
        .expect_output("command2")
        .expect_output("conclude")
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

#[test]
fn reference_is_executed_first() {
    ExecutionOrderTest::new()
        .arg("--runs=1")
        .reference("reference")
        .command("command 1")
        .command("command 2")
        .expect_output("reference")
        .expect_output("command 1")
        .expect_output("command 2")
        .run();
}

#[test]
fn reference_is_executed_first_parameter_value() {
    ExecutionOrderTest::new()
        .arg("--runs=2")
        .reference("reference")
        .arg("--parameter-list")
        .arg("number")
        .arg("1,2,3")
        .command("command {number}")
        .expect_output("reference")
        .expect_output("reference")
        .expect_output("command 1")
        .expect_output("command 1")
        .expect_output("command 2")
        .expect_output("command 2")
        .expect_output("command 3")
        .expect_output("command 3")
        .run();
}

#[test]
fn reference_is_executed_separately_from_commands() {
    ExecutionOrderTest::new()
        .arg("--runs=1")
        .reference("command 1")
        .command("command 1")
        .command("command 2")
        .expect_output("command 1")
        .expect_output("command 1")
        .expect_output("command 2")
        .run();
}

#[test]
fn setup_prepare_reference_conclude_cleanup_combined() {
    ExecutionOrderTest::new()
        .arg("--warmup=1")
        .arg("--runs=2")
        .setup("setup")
        .prepare("prepare")
        .reference("reference")
        .command("command1")
        .command("command2")
        .conclude("conclude")
        .cleanup("cleanup")
        // reference
        .expect_output("setup")
        .expect_output("prepare")
        .expect_output("reference")
        .expect_output("conclude")
        .expect_output("prepare")
        .expect_output("reference")
        .expect_output("conclude")
        .expect_output("prepare")
        .expect_output("reference")
        .expect_output("conclude")
        .expect_output("cleanup")
        // 1
        .expect_output("setup")
        .expect_output("prepare")
        .expect_output("command1")
        .expect_output("conclude")
        .expect_output("prepare")
        .expect_output("command1")
        .expect_output("conclude")
        .expect_output("prepare")
        .expect_output("command1")
        .expect_output("conclude")
        .expect_output("cleanup")
        // 2
        .expect_output("setup")
        .expect_output("prepare")
        .expect_output("command2")
        .expect_output("conclude")
        .expect_output("prepare")
        .expect_output("command2")
        .expect_output("conclude")
        .expect_output("prepare")
        .expect_output("command2")
        .expect_output("conclude")
        .expect_output("cleanup")
        .run();
}

#[test]
fn setup_separate_prepare_separate_conclude_cleanup_combined() {
    ExecutionOrderTest::new()
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
        // 1
        .expect_output("setup")
        .expect_output("prepare1")
        .expect_output("command1")
        .expect_output("conclude1")
        .expect_output("prepare1")
        .expect_output("command1")
        .expect_output("conclude1")
        .expect_output("prepare1")
        .expect_output("command1")
        .expect_output("conclude1")
        .expect_output("cleanup")
        // 2
        .expect_output("setup")
        .expect_output("prepare2")
        .expect_output("command2")
        .expect_output("conclude2")
        .expect_output("prepare2")
        .expect_output("command2")
        .expect_output("conclude2")
        .expect_output("prepare2")
        .expect_output("command2")
        .expect_output("conclude2")
        .expect_output("cleanup")
        .run();
}

#[test]
fn setup_separate_prepare_reference_separate_conclude_cleanup_combined() {
    ExecutionOrderTest::new()
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
        // reference
        .expect_output("setup")
        .expect_output("prepareref")
        .expect_output("reference")
        .expect_output("concluderef")
        .expect_output("prepareref")
        .expect_output("reference")
        .expect_output("concluderef")
        .expect_output("prepareref")
        .expect_output("reference")
        .expect_output("concluderef")
        .expect_output("cleanup")
        // 1
        .expect_output("setup")
        .expect_output("prepare1")
        .expect_output("command1")
        .expect_output("conclude1")
        .expect_output("prepare1")
        .expect_output("command1")
        .expect_output("conclude1")
        .expect_output("prepare1")
        .expect_output("command1")
        .expect_output("conclude1")
        .expect_output("cleanup")
        // 2
        .expect_output("setup")
        .expect_output("prepare2")
        .expect_output("command2")
        .expect_output("conclude2")
        .expect_output("prepare2")
        .expect_output("command2")
        .expect_output("conclude2")
        .expect_output("prepare2")
        .expect_output("command2")
        .expect_output("conclude2")
        .expect_output("cleanup")
        .run();
}
