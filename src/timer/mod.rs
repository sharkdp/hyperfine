pub mod wall_clock_timer;

#[cfg(windows)]
mod windows_timer;

#[cfg(not(windows))]
mod unix_timer;

use crate::util::units::Second;

use std::process::{Command, ExitStatus};

use anyhow::Result;

#[derive(Debug, Copy, Clone)]
struct CPUTimes {
    /// Total amount of time spent executing in user mode
    pub user_usec: i64,

    /// Total amount of time spent executing in kernel mode
    pub system_usec: i64,
}

/// Used to indicate the result of running a command
#[derive(Debug, Copy, Clone)]
pub struct ExecuteResult {
    /// The amount of user time the process used
    pub user_time: Second,

    /// The amount of cpu time the process used
    pub system_time: Second,

    /// The exit status of the process
    pub status: ExitStatus,
}

/// Execute the given command and return a timing summary
#[cfg(not(windows))]
pub fn execute_and_measure(mut command: Command) -> Result<ExecuteResult> {
    let cpu_timer = self::unix_timer::CPUTimer::start();
    let status = command.status()?;
    let (user_time, system_time) = cpu_timer.stop();

    Ok(ExecuteResult {
        user_time,
        system_time,
        status,
    })
}

/// Execute the given command and return a timing summary
#[cfg(windows)]
pub fn execute_and_measure(mut command: Command) -> Result<ExecuteResult> {
    let mut child = command.spawn()?;
    let cpu_timer = self::windows_timer::CPUTimer::start_for_process(&child);
    let status = child.wait()?;

    let (user_time, system_time) = cpu_timer.stop();
    Ok(ExecuteResult {
        user_time,
        system_time,
        status,
    })
}
