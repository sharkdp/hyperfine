use std::process::{Command, ExitStatus, Stdio};

use crate::options::Shell;
use crate::timer::get_cpu_timer;

use anyhow::{Context, Error, Result};

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
    shell: &Shell,
) -> Result<ExecuteResult> {
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
    shell: &Shell,
) -> Result<ExecuteResult> {
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
    shell: &Shell,
) -> Result<std::process::ExitStatus> {
    prepare_shell_command(command, shell, &["-c"])?
        .env(
            "HYPERFINE_RANDOMIZED_ENVIRONMENT_OFFSET",
            "X".repeat(rand::random::<usize>() % 4096usize),
        )
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .status()
        .with_context(|| format!("Failed to run command '{}'", command))
}

/// Run a Windows shell command using `cmd.exe /C`
#[cfg(windows)]
fn run_shell_command(
    stdout: Stdio,
    stderr: Stdio,
    command: &str,
    shell: &Shell,
) -> Result<std::process::Child> {
    prepare_shell_command(command, shell, &["/C"])?
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .with_context(|| format!("Failed to run command '{}'", command))
}

fn prepare_shell_command(
    command: &str,
    shell: &Shell,
    shell_extra_args: &[&str],
) -> Result<std::process::Command> {
    match shell.command() {
        Some(mut shell_command) => {
            shell_command.args(shell_extra_args).arg(command);
            Ok(shell_command)
        }
        None => {
            let mut tokens = shell_words::split(command)
                .with_context(|| format!("Failed to parse command '{}", command))?
                .into_iter();
            if let Some(exec) = tokens.next() {
                let mut shell_command = Command::new(exec);
                shell_command.args(tokens);
                Ok(shell_command)
            } else {
                Err(Error::msg("Failed to parse empty command"))
            }
        }
    }
}
