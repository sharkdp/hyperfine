use crate::hyperfine::units::{MebiByte, Second};

#[derive(Debug, Copy, Clone)]
pub struct CPUTimes {
    /// Total amount of time spent executing in user mode
    pub user_usec: i64,

    /// Total amount of time spent executing in kernel mode
    pub system_usec: i64,

    /// Maximum resident set in bytes
    pub max_rss_byte: i64,
}

#[derive(Debug, Copy, Clone)]
pub struct CPUInterval {
    /// Total amount of time spent executing in user mode
    pub user: Second,

    /// Total amount of time spent executing in kernel mode
    pub system: Second,

    /// Maximum resident set
    pub max_rss: MebiByte,
}
