use std::process::{ExitStatus, Stdio};

use crate::options::Shell;
use crate::util::randomized_environment_offset;

use anyhow::{Context, Result};

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
#[cfg(not(windows))]
pub fn execute_and_measure(
    stdout: Stdio,
    stderr: Stdio,
    command: &str,
    shell: &Shell,
) -> Result<ExecuteResult> {
    let cpu_timer = crate::timer::unix_timer::CPUTimer::start();

    let status = shell
        .command()
        .arg("-c")
        .arg(command)
        .env(
            "HYPERFINE_RANDOMIZED_ENVIRONMENT_OFFSET",
            randomized_environment_offset::value(),
        )
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .status()
        .with_context(|| format!("Failed to run command '{}'", command))?;

    let (user_time, system_time) = cpu_timer.stop();

    Ok(ExecuteResult {
        user_time,
        system_time,
        status,
    })
}

/// Execute the given command and return a timing summary
#[cfg(windows)]
pub fn execute_and_measure(
    stdout: Stdio,
    stderr: Stdio,
    command: &str,
    shell: &Shell,
) -> Result<ExecuteResult> {
    let mut child = shell
        .command()
        .arg("/C")
        .arg(command)
        .env(
            "HYPERFINE_RANDOMIZED_ENVIRONMENT_OFFSET",
            randomized_environment_offset::value(),
        )
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .with_context(|| format!("Failed to run command '{}'", command))?;
    let cpu_timer = crate::timer::windows_timer::CPUTimer::start_for_process(&child);
    let status = child.wait()?;

    let (user_time, system_time) = cpu_timer.stop();
    Ok(ExecuteResult {
        user_time,
        system_time,
        status,
    })
}
