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

/// Read CPU execution times ('user' and 'system')
pub fn get_cpu_times() -> CPUTimes {
    let result: rusage = unsafe {
        let mut buf = mem::zeroed();
        let success = getrusage(RUSAGE_CHILDREN, &mut buf);
        assert_eq!(0, success);
        buf
    };

    const MICROSEC_PER_SEC: i64 = 1000 * 1000;

    CPUTimes {
        user_usec: i64::from(result.ru_utime.tv_sec) * MICROSEC_PER_SEC
            + i64::from(result.ru_utime.tv_usec),
        system_usec: i64::from(result.ru_stime.tv_sec) * MICROSEC_PER_SEC
            + i64::from(result.ru_stime.tv_usec),
    }
}

/// Compute the time intervals in between two `CPUTimes` snapshots
pub fn cpu_time_interval(start: &CPUTimes, end: &CPUTimes) -> CPUInterval {
    CPUInterval {
        user: ((end.user_usec - start.user_usec) as f64) * 1e-6,
        system: ((end.system_usec - start.system_usec) as f64) * 1e-6,
    }
}

#[test]
fn test_cpu_time_interval() {
    let t_a = CPUTimes {
        user_usec: 12345,
        system_usec: 54321,
    };

    let t_b = CPUTimes {
        user_usec: 20000,
        system_usec: 70000,
    };

    let t_zero = cpu_time_interval(&t_a, &t_a);
    assert_eq!(0.0, t_zero.user);
    assert_eq!(0.0, t_zero.system);

    let t_ab = cpu_time_interval(&t_a, &t_b);
    assert_relative_eq!(0.007655, t_ab.user);
    assert_relative_eq!(0.015679, t_ab.system);

    let t_ba = cpu_time_interval(&t_b, &t_a);
    assert_relative_eq!(-0.007655, t_ba.user);
    assert_relative_eq!(-0.015679, t_ba.system);
}
