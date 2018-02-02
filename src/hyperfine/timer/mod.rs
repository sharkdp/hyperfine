pub mod wallclocktimer;
pub mod cputimer;

/// Defines a general timer with start/stop functionality.
pub trait Timer {
    type Result;

    fn start() -> Self;
    fn stop(&self) -> Self::Result;
}
