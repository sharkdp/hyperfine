#![cfg(not(windows))]

use std::convert::TryFrom;
use std::mem;

use crate::timer::CPUTimes;
use crate::util::units::Second;

#[derive(Debug, Copy, Clone)]
pub struct CPUInterval {
    /// Total amount of time spent executing in user mode
    pub user: Second,

    /// Total amount of time spent executing in kernel mode
    pub system: Second,
}

pub struct CPUTimer {
    start_cpu: CPUTimes,
}

impl CPUTimer {
    pub fn start() -> Self {
        CPUTimer {
            start_cpu: get_cpu_times(),
        }
    }

    pub fn stop(&self) -> (Second, Second, u64) {
        let end_cpu = get_cpu_times();
        let cpu_interval = cpu_time_interval(&self.start_cpu, &end_cpu);
        (
            cpu_interval.user,
            cpu_interval.system,
            end_cpu.memory_usage_byte,
        )
    }
}

/// Read CPU execution times ('user' and 'system')
fn get_cpu_times() -> CPUTimes {
    use libc::{getrusage, rusage, RUSAGE_CHILDREN};

    let result: rusage = unsafe {
        let mut buf = mem::zeroed();
        let success = getrusage(RUSAGE_CHILDREN, &mut buf);
        assert_eq!(0, success);
        buf
    };

    const MICROSEC_PER_SEC: i64 = 1000 * 1000;

    // Linux and *BSD return the value in KibiBytes, Darwin flavors in bytes
    let max_rss_byte = if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
        result.ru_maxrss
    } else {
        result.ru_maxrss * 1024
    };

    #[allow(clippy::useless_conversion)]
    CPUTimes {
        user_usec: i64::from(result.ru_utime.tv_sec) * MICROSEC_PER_SEC
            + i64::from(result.ru_utime.tv_usec),
        system_usec: i64::from(result.ru_stime.tv_sec) * MICROSEC_PER_SEC
            + i64::from(result.ru_stime.tv_usec),
        memory_usage_byte: u64::try_from(max_rss_byte).unwrap_or(0),
    }
}

/// Compute the time intervals in between two `CPUTimes` snapshots
fn cpu_time_interval(start: &CPUTimes, end: &CPUTimes) -> CPUInterval {
    CPUInterval {
        user: ((end.user_usec - start.user_usec) as f64) * 1e-6,
        system: ((end.system_usec - start.system_usec) as f64) * 1e-6,
    }
}

#[cfg(test)]
use approx::assert_relative_eq;

#[test]
fn test_cpu_time_interval() {
    let t_a = CPUTimes {
        user_usec: 12345,
        system_usec: 54321,
        memory_usage_byte: 0,
    };

    let t_b = CPUTimes {
        user_usec: 20000,
        system_usec: 70000,
        memory_usage_byte: 0,
    };

    let t_zero = cpu_time_interval(&t_a, &t_a);
    assert!(t_zero.user.abs() < f64::EPSILON);
    assert!(t_zero.system.abs() < f64::EPSILON);

    let t_ab = cpu_time_interval(&t_a, &t_b);
    assert_relative_eq!(0.007655, t_ab.user);
    assert_relative_eq!(0.015679, t_ab.system);

    let t_ba = cpu_time_interval(&t_b, &t_a);
    assert_relative_eq!(-0.007655, t_ba.user);
    assert_relative_eq!(-0.015679, t_ba.system);
}
