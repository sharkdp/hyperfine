use std::io;
use std::process::{Command, ExitStatus, Stdio};

/// Run a standard shell command
#[cfg(not(target_os = "windows"))]
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
#[cfg(target_os = "windows")]
pub fn run_shell_command(command: &str) -> io::Result<ExitStatus> {
    Command::new("cmd")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
}
