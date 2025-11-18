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

    /// Maximum amount of memory used, in bytes
    pub memory_usage_byte: u64,

    /// Number of voluntary context switches
    pub voluntary_context_switches: u64,

    /// Number of involuntary context switches
    pub context_switches: u64,

    /// Number of times the filesystem had to perform input.
    pub filesystem_input: u64,

    /// Number of times the filesystem had to perform output.
    pub filesystem_output: u64,

    /// Number of minor page faults
    pub minor_page_faults: u64,

    /// Number of major page faults
    pub major_page_faults: u64,
}
