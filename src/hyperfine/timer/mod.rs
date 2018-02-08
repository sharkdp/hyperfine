mod internal;

pub mod wallclocktimer;

cfg_if! {
    if #[cfg(windows)] {
        mod windows_timer;
        pub use self::windows_timer::get_cpu_timer;
    } else {
        mod unix_timer;
        pub use self::unix_timer::get_cpu_timer;
    }
}
use std::process::Child;

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
