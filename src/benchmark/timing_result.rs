use crate::util::units::Second;

/// Results from timing a single command
#[derive(Debug, Default, Copy, Clone)]
pub struct TimingResult {
    /// Wall clock time
    pub time_real: Second,

    /// Time spent in user mode
    pub time_user: Second,

    /// Time spent in kernel mode
    pub time_system: Second,

    /// Amount of memory used
    pub max_rss_byte: Option<i64>,
}
