use crate::util::units::Second;

/// Results from timing a single command
#[derive(Debug, Default, Copy, Clone)]
pub struct TimingResult {
    /// Wall clock time (post-exec on Linux, for reporting)
    pub time_real: Second,

    /// Full wall clock time including fork overhead (for scheduling)
    pub time_real_full: Second,

    /// Time spent in user mode
    pub time_user: Second,

    /// Time spent in kernel mode
    pub time_system: Second,

    /// Maximum amount of memory used, in bytes
    pub memory_usage_byte: u64,
}
