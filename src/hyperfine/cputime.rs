use libc::{getrusage, rusage, RUSAGE_CHILDREN};
use std::mem;

use hyperfine::internal::Second;

#[derive(Debug, Copy, Clone)]
pub struct CPUTimes {
    /// Total amount of time spent executing in user mode
    user_usec: i64,

    /// Total amount of time spent executing in kernel mode
    system_usec: i64,
}

#[derive(Debug, Copy, Clone)]
pub struct CPUInterval {
    /// Total amount of time spent executing in user mode
    pub user: Second,

    /// Total amount of time spent executing in kernel mode
    pub system: Second,
}

/// Read CPU execution times (user and system time) from kernel
pub fn get_cpu_times() -> CPUTimes {
    let result: rusage = unsafe {
        let mut buf = mem::zeroed();
        let success = getrusage(RUSAGE_CHILDREN, &mut buf);
        assert_eq!(0, success);
        buf
    };

    const USEC_PER_SEC: i64 = 1000 * 1000;

    CPUTimes {
        user_usec: i64::from(result.ru_utime.tv_sec) * USEC_PER_SEC
            + i64::from(result.ru_utime.tv_usec),
        system_usec: i64::from(result.ru_stime.tv_sec) * USEC_PER_SEC
            + i64::from(result.ru_stime.tv_usec),
    }
}

/// Compute the time intervals in between two CPUTimes snapshots
pub fn get_cpu_interval(start: &CPUTimes, end: &CPUTimes) -> CPUInterval {
    CPUInterval {
        user: ((end.user_usec - start.user_usec) as f64) * 1e-6,
        system: ((end.system_usec - start.system_usec) as f64) * 1e-6,
    }
}
