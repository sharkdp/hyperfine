use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

pub fn hyperfine_raw_command() -> Command {
    let mut cmd = Command::cargo_bin("hyperfine").unwrap();
    cmd.current_dir("tests/");
    cmd
}

pub fn hyperfine() -> assert_cmd::Command {
    assert_cmd::Command::from_std(hyperfine_raw_command())
}

pub fn hyperfine_shell() -> assert_cmd::Command {
    let mut cmd = hyperfine();
    cmd.arg("--shell=default");
    cmd
}
