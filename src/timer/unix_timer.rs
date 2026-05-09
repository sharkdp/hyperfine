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

    pub fn stop(&self) -> (Second, Second, u64, u64, u64, u64, u64, u64) {
        let end_cpu = get_cpu_times();
        let cpu_interval = cpu_time_interval(&self.start_cpu, &end_cpu);

        (
            cpu_interval.user,
            cpu_interval.system,
            end_cpu.voluntary_context_switches - self.start_cpu.voluntary_context_switches,
            end_cpu.context_switches - self.start_cpu.context_switches,
            end_cpu.filesystem_input - self.start_cpu.filesystem_input,
            end_cpu.filesystem_output - self.start_cpu.filesystem_output,
            end_cpu.minor_page_faults - self.start_cpu.minor_page_faults,
            end_cpu.major_page_faults - self.start_cpu.major_page_faults,
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

    let voluntary_context_switches =
        u64::try_from(result.ru_nvcsw).expect("should never be negative");
    let context_switches = u64::try_from(result.ru_nivcsw).expect("should never be negative");
    let filesystem_input = u64::try_from(result.ru_inblock).expect("should never be negative");
    let filesystem_output = u64::try_from(result.ru_oublock).expect("should never be negative");
    let minor_page_faults = u64::try_from(result.ru_minflt).expect("should never be negative");
    let major_page_faults = u64::try_from(result.ru_majflt).expect("should never be negative");

    #[allow(clippy::useless_conversion)]
    CPUTimes {
        user_usec: i64::from(result.ru_utime.tv_sec) * MICROSEC_PER_SEC
            + i64::from(result.ru_utime.tv_usec),
        system_usec: i64::from(result.ru_stime.tv_sec) * MICROSEC_PER_SEC
            + i64::from(result.ru_stime.tv_usec),
        voluntary_context_switches,
        context_switches,
        filesystem_input,
        filesystem_output,
        minor_page_faults,
        major_page_faults,
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
        voluntary_context_switches: 0,
        context_switches: 0,
        filesystem_input: 0,
        filesystem_output: 0,
        minor_page_faults: 0,
        major_page_faults: 0,
    };

    let t_b = CPUTimes {
        user_usec: 20000,
        system_usec: 70000,
        voluntary_context_switches: 0,
        context_switches: 0,
        filesystem_input: 0,
        filesystem_output: 0,
        minor_page_faults: 0,
        major_page_faults: 0,
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
