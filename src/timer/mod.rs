use std::process::Child;

use crate::units::Second;

pub mod wallclocktimer;

#[cfg(windows)]
mod windows_timer;
#[cfg(windows)]
pub use self::windows_timer::get_cpu_timer;

#[cfg(not(windows))]
mod unix_timer;
#[cfg(not(windows))]
pub use self::unix_timer::get_cpu_timer;

/// Defines start functionality of a timer.
pub trait TimerStart {
    fn start() -> Self;
    fn start_for_process(process: &Child) -> Self;
}

/// Defines stop functionality of a timer.
pub trait TimerStop {
    type Result;
    fn stop(&self) -> Self::Result;
}

#[derive(Debug, Copy, Clone)]
pub struct CPUTimes {
    /// Total amount of time spent executing in user mode
    pub user_usec: i64,

    /// Total amount of time spent executing in kernel mode
    pub system_usec: i64,
}

#[derive(Debug, Copy, Clone)]
pub struct CPUInterval {
    /// Total amount of time spent executing in user mode
    pub user: Second,

    /// Total amount of time spent executing in kernel mode
    pub system: Second,
}
