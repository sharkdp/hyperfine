use std::process::ExitStatus;

use crate::util::units::Second;

/// Performance measurements and exit status from running a single command
#[derive(Debug, Default, Copy, Clone)]
pub struct Measurement {
    /// Wall clock time
    pub time_wall_clock: Second,

    /// Time spent in user mode
    pub time_user: Second,

    /// Time spent in kernel mode
    pub time_system: Second,

    /// Maximum amount of memory used, in bytes
    pub peak_memory_usage: u64,

    /// The exit status of the process
    pub exit_status: ExitStatus,
}
