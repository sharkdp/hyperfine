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
    /// number of voluntary context swithcing
    pub voluntary_cs: u64,
    /// number of involuntary context swithcing
    pub involuntary_cs: u64,

    ///  number of io read operations
    pub io_read_ops: u64,

    /// number of io write operations
    pub io_write_ops: u64,
}
