mod wall_clock_timer;

#[cfg(windows)]
mod windows_timer;

#[cfg(not(windows))]
mod unix_timer;

use crate::util::units::Second;
use wall_clock_timer::WallClockTimer;

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
pub struct TimerResult {
    pub time_real: Second,
    pub time_user: Second,
    pub time_system: Second,

    /// The exit status of the process
    pub status: ExitStatus,
}

/// Execute the given command and return a timing summary
pub fn execute_and_measure(mut command: Command) -> Result<TimerResult> {
    let wallclock_timer = WallClockTimer::start();

    #[cfg(not(windows))]
    let ((time_user, time_system), status) = {
        let cpu_timer = self::unix_timer::CPUTimer::start();
        let status = command.status()?;
        (cpu_timer.stop(), status)
    };

    #[cfg(windows)]
    let ((time_user, time_system), status) = {
        let mut child = command.spawn()?;
        let cpu_timer = self::windows_timer::CPUTimer::start_for_process(&child);
        let status = child.wait()?;

        (cpu_timer.stop(), status)
    };

    let time_real = wallclock_timer.stop();

    Ok(TimerResult {
        time_real,
        time_user,
        time_system,
        status,
    })
}
