use std::io;
use std::mem::MaybeUninit;
use std::process::Child;

use crate::benchmark::measurement::HardwareCounters;

/// Wait for exit without reaping: proc_pid_rusage can still inspect the zombie,
/// and the existing wait4 call can subsequently collect CPU times and peak RSS.
pub fn wait(child: &mut Child) -> io::Result<()> {
    drop(child.stdin.take());
    let mut info = MaybeUninit::uninit();
    loop {
        // SAFETY: info points to writable siginfo_t storage. WNOWAIT preserves the child.
        let result = unsafe {
            libc::waitid(
                libc::P_PID,
                child.id(),
                info.as_mut_ptr(),
                libc::WEXITED | libc::WNOWAIT,
            )
        };
        if result == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

pub fn read(pid: u32) -> HardwareCounters {
    let mut usage = MaybeUninit::<libc::rusage_info_v4>::zeroed();
    // SAFETY: the buffer has the layout and size requested by RUSAGE_INFO_V4.
    // Despite the pointer-to-pointer signature, this API writes directly into the buffer.
    let result = unsafe {
        libc::proc_pid_rusage(
            pid as libc::pid_t,
            libc::RUSAGE_INFO_V4,
            usage.as_mut_ptr().cast(),
        )
    };
    if result != 0 {
        return HardwareCounters::default();
    }
    // SAFETY: the call succeeded and initialized the buffer.
    let usage = unsafe { usage.assume_init() };
    HardwareCounters {
        // Like macOS time(1), omit zero counters: the API can succeed on systems
        // where instruction/cycle accounting is unavailable.
        cpu_cycles: (usage.ri_cycles != 0).then_some(usage.ri_cycles),
        instructions: (usage.ri_instructions != 0).then_some(usage.ri_instructions),
        ..Default::default()
    }
}
