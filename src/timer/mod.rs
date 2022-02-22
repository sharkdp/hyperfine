use crate::util::units::Second;

pub mod wallclocktimer;

#[cfg(windows)]
pub mod windows_timer;

#[cfg(not(windows))]
pub mod unix_timer;

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
