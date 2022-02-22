use std::process::{Command, ExitStatus};

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
pub fn execute_and_measure(mut command: Command, error_message: &str) -> Result<ExecuteResult> {
    let cpu_timer = crate::timer::unix_timer::CPUTimer::start();
    let status = command
        .status()
        .with_context(|| error_message.to_string())?;
    let (user_time, system_time) = cpu_timer.stop();

    Ok(ExecuteResult {
        user_time,
        system_time,
        status,
    })
}

/// Execute the given command and return a timing summary
#[cfg(windows)]
pub fn execute_and_measure(mut command: Command, error_message: &str) -> Result<ExecuteResult> {
    let mut child = command.spawn().with_context(|| error_message.to_string())?;
    let cpu_timer = crate::timer::windows_timer::CPUTimer::start_for_process(&child);
    let status = child.wait()?;

    let (user_time, system_time) = cpu_timer.stop();
    Ok(ExecuteResult {
        user_time,
        system_time,
        status,
    })
}
