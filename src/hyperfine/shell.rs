use std::io;
use std::process::{Command, Stdio};

#[cfg(not(windows))]
use std::process::ExitStatus;

#[cfg(windows)]
use std::process::Child;

/// Run a standard shell command
#[cfg(not(windows))]
pub fn run_shell_command(command: &str) -> io::Result<ExitStatus> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
}

/// Run a Windows shell command using cmd.exe
#[cfg(windows)]
pub fn run_shell_command(command: &str) -> io::Result<Child> {
    Command::new("cmd")
        .arg("/C")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}
