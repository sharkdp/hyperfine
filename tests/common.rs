use std::process::Command;

pub fn hyperfine_raw_command() -> Command {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("hyperfine"));
    cmd.current_dir("tests/");
    cmd
}

pub fn hyperfine() -> assert_cmd::Command {
    assert_cmd::Command::from_std(hyperfine_raw_command())
}
