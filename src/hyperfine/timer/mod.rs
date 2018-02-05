mod internal;

pub mod wallclocktimer;

#[cfg(not(windows))]
pub mod unix_timer;

#[cfg(windows)]
pub mod windows_timer;

/// Defines a general timer with start/stop functionality.
pub trait Timer {
    type Result;

    fn start() -> Self;
    fn stop(&self) -> Self::Result;
}
