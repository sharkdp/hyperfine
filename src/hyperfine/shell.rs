use std;
use std::io;
use std::process::{Command, ExitStatus, Stdio};

use crate::hyperfine::timer::get_cpu_timer;

/// Used to indicate the result of running a command
#[derive(Debug, Copy, Clone)]
pub struct ExecuteResult {
    /// The amount of user time the process used
    pub user_time: f64,

    /// The amount of cpu time the process used
    pub system_time: f64,

    /// The exit status of the process
    pub status: ExitStatus,
}

/// Execute the given command and return a timing summary
#[cfg(windows)]
pub fn execute_and_time(
    stdout: Stdio,
    stderr: Stdio,
    command: &str,
    shell: &str,
) -> io::Result<ExecuteResult> {
    let mut child = run_shell_command(stdout, stderr, command, shell)?;
    let cpu_timer = get_cpu_timer(&child);
    let status = child.wait()?;

    let (user_time, system_time) = cpu_timer.stop();
    Ok(ExecuteResult {
        user_time,
        system_time,
        status,
    })
}

/// Execute the given command and return a timing summary
#[cfg(not(windows))]
pub fn execute_and_time(
    stdout: Stdio,
    stderr: Stdio,
    command: &str,
    shell: &str,
) -> io::Result<ExecuteResult> {
    let cpu_timer = get_cpu_timer();

    let status = run_shell_command(stdout, stderr, command, shell)?;

    let (user_time, system_time) = cpu_timer.stop();

    Ok(ExecuteResult {
        user_time,
        system_time,
        status,
    })
}

/// Run a standard shell command using `sh -c`
#[cfg(not(windows))]
fn run_shell_command(
    stdout: Stdio,
    stderr: Stdio,
    command: &str,
    shell: &str,
) -> io::Result<std::process::ExitStatus> {
    Command::new(shell)
        .arg("-c")
        .arg(command)
        .env(
            "HYPERFINE_RANDOMIZED_ENVIRONMENT_OFFSET",
            "X".repeat(rand::random::<usize>() % 4096usize),
        )
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .status()
}

/// Run a Windows shell command using `cmd.exe /C`
#[cfg(windows)]
fn run_shell_command(
    stdout: Stdio,
    stderr: Stdio,
    command: &str,
    shell: &str,
) -> io::Result<std::process::Child> {
    Command::new(shell)
        .arg("/C")
        .arg(command)
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
}
