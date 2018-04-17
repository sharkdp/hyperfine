use std::process::{Command, ExitStatus, Stdio};
use std::{self, io, str};

use hyperfine::timer::get_cpu_timer;

/// Used to indicate the result of running a command
#[derive(Debug, Clone)]
pub struct ExecuteResult {
    /// The amount of user time the process used
    pub user_time: f64,

    /// The amount of cpu time the process used
    pub system_time: f64,

    /// The exit status of the process
    pub status: ExitStatus,

    /// Captured standard output
    pub stdout: Option<String>,

    /// Captured standard error
    pub stderr: Option<String>,
}

/// Execute the given command and return a timing summary
#[cfg(windows)]
pub fn execute_and_time(command: &str, capture: bool) -> io::Result<ExecuteResult> {
    let mut child = run_shell_command(command, capture)?;
    let cpu_timer = get_cpu_timer(&child);
    let status = child.wait()?;

    let (stdout, stderr) = (None, None);

    let (user_time, system_time) = cpu_timer.stop();
    Ok(ExecuteResult {
        user_time,
        system_time,
        status,
        stdout,
        stderr,
    })
}

/// Execute the given command and return a timing summary
#[cfg(not(windows))]
pub fn execute_and_time(command: &str, capture: bool) -> io::Result<ExecuteResult> {
    let cpu_timer = get_cpu_timer();

    let out = run_shell_command(command, capture)?;

    let status = out.status;

    let (stdout, stderr) = if capture {
        (String::from_utf8(out.stdout).ok(), String::from_utf8(out.stderr).ok())
    } else {
        (None, None)
    };

    let (user_time, system_time) = cpu_timer.stop();

    Ok(ExecuteResult {
        user_time,
        system_time,
        status,
        stdout,
        stderr,
    })
}

/// Run a standard shell command
#[cfg(not(windows))]
fn run_shell_command(command: &str, capture: bool) -> io::Result<std::process::Output> {
    let (stdout, stderr) = if capture {
        (Stdio::piped(), Stdio::piped())
    } else {
        (Stdio::null(), Stdio::null())
    };

    Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .output()
}

/// Run a Windows shell command using cmd.exe
#[cfg(windows)]
fn run_shell_command(command: &str, capture: bool) -> io::Result<std::process::Child> {
    let (stdout, stderr) = if capture {
        (Stdio::piped(), Stdio::piped())
    } else {
        (Stdio::null(), Stdio::null())
    };

    Command::new("cmd")
        .arg("/C")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
}
