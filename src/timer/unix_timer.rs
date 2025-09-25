#![cfg(not(windows))]

use std::io;
use std::mem::MaybeUninit;
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, ExitStatus};

use anyhow::Result;

use crate::quantity::{byte, kibibyte, microsecond, second, Information, Time};

#[derive(Debug, Copy, Clone)]
struct ResourceUsage {
    /// Total amount of time spent executing in user mode
    pub time_user: Time,

    /// Total amount of time spent executing in kernel mode
    pub time_system: Time,

    /// Maximum amount of memory used by the process, in bytes
    pub memory_usage: Information,
}

#[allow(clippy::useless_conversion)]
fn convert_timeval(tv: libc::timeval) -> Time {
    let sec = tv.tv_sec as f64;
    let usec = tv.tv_usec as f64;

    Time::new::<second>(sec) + Time::new::<microsecond>(usec)
}

#[allow(clippy::useless_conversion)]
fn wait4(mut child: Child) -> io::Result<(ExitStatus, ResourceUsage)> {
    drop(child.stdin.take());

    let pid = child.id() as i32;
    let mut status = 0;
    let mut rusage = MaybeUninit::zeroed();

    let result = unsafe { libc::wait4(pid, &mut status, 0, rusage.as_mut_ptr()) };

    if result < 0 {
        Err(io::Error::last_os_error())
    } else {
        let rusage = unsafe { rusage.assume_init() };

        let memory_usage_byte = if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
            // Linux and *BSD return the value in KibiBytes, Darwin flavors in bytes
            // A f64 can represent integers up to 2^53 exactly, so we can represent
            // all values up to 2^53 bytes = 8 PiB exactly. Beyond that, our precision
            // is larger than 1 byte, but that's considered acceptable.
            Information::new::<byte>(rusage.ru_maxrss as f64)
        } else {
            Information::new::<kibibyte>(rusage.ru_maxrss as f64)
        };

        Ok((
            ExitStatus::from_raw(status),
            ResourceUsage {
                time_user: convert_timeval(rusage.ru_utime),
                time_system: convert_timeval(rusage.ru_stime),
                memory_usage: memory_usage_byte.into(),
            },
        ))
    }
}

pub struct CPUTimer {}

impl CPUTimer {
    pub fn start() -> Self {
        Self {}
    }

    pub fn stop(&self, child: Child) -> Result<(Time, Time, Information, ExitStatus)> {
        let (status, usage) = wait4(child)?;
        Ok((
            usage.time_user,
            usage.time_system,
            usage.memory_usage,
            status,
        ))
    }
}
