mod internal;

pub mod wallclocktimer;

#[cfg(not(windows))]
mod unix_timer;

#[cfg(not(windows))]
pub use self::unix_timer::get_cpu_timer;

#[cfg(windows)]
mod windows_timer;

#[cfg(windows)]
pub use self::windows_timer::get_cpu_timer;

use std::process::Child;

pub trait Timer
where
    Self: TimerStart + TimerStop,
{
}

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
