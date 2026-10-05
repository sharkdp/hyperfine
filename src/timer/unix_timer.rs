#![cfg(not(windows))]

use std::convert::TryFrom;
use std::io;
use std::mem::MaybeUninit;
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, ExitStatus};

use anyhow::Result;

use crate::util::units::Second;

#[derive(Debug, Copy, Clone)]
struct ResourceUsage {
    /// Total amount of time spent executing in user mode
    pub user_usec: Second,

    /// Total amount of time spent executing in kernel mode
    pub system_usec: Second,

    /// Maximum amount of memory used by the process, in bytes
    pub memory_usage_byte: i64,
}

#[allow(clippy::useless_conversion)]
fn timeval_to_seconds(tv: libc::timeval) -> Second {
    const MICROSEC_PER_SEC: i64 = 1000 * 1000;
    (i64::from(tv.tv_sec) * MICROSEC_PER_SEC + i64::from(tv.tv_usec)) as f64 * 1e-6
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
            rusage.ru_maxrss
        } else {
            rusage.ru_maxrss * 1024
        };

        Ok((
            ExitStatus::from_raw(status),
            ResourceUsage {
                user_usec: timeval_to_seconds(rusage.ru_utime),
                system_usec: timeval_to_seconds(rusage.ru_stime),
                memory_usage_byte: memory_usage_byte.into(),
            },
        ))
    }
}

pub struct CPUTimer {}

impl CPUTimer {
    pub fn start() -> Self {
        Self {}
    }

    pub fn stop(&self, child: Child) -> Result<(ExitStatus, Second, Second, u64)> {
        let (status, usage) = wait4(child)?;
        Ok((
            status,
            usage.user_usec,
            usage.system_usec,
            u64::try_from(usage.memory_usage_byte).unwrap_or(0),
        ))
    }
}
