mod wall_clock_timer;

#[cfg(windows)]
mod windows_timer;

#[cfg(not(windows))]
mod unix_timer;

#[cfg(target_os = "linux")]
use nix::fcntl::{splice, SpliceFFlags};
#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(target_os = "linux")]
use std::os::fd::AsFd;

#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::CREATE_SUSPENDED;

use crate::util::units::Second;
use wall_clock_timer::WallClockTimer;

use std::io::Read;
use std::process::{ChildStdout, Command, ExitStatus};

use anyhow::Result;

#[cfg(not(windows))]
#[derive(Debug, Copy, Clone)]
struct CPUTimes {
    /// Total amount of time spent executing in user mode
    pub user_usec: i64,

    /// Total amount of time spent executing in kernel mode
    pub system_usec: i64,

    /// Maximum amount of memory used by the process, in bytes
    pub memory_usage_byte: u64,
}

/// Used to indicate the result of running a command
#[derive(Debug, Copy, Clone)]
pub struct TimerResult {
    pub time_real: Second,
    pub time_user: Second,
    pub time_system: Second,
    pub memory_usage_byte: u64,
   pub voluntary_cs:i32,
   pub involuntary_cs :i32,
   pub io_read_ops: u64,
   pub io_write_ops :u64,
    /// The exit status of the process
    pub status: ExitStatus,
}

/// Discard the output of a child process.
fn discard(output: ChildStdout) {
    const CHUNK_SIZE: usize = 64 << 10;

    #[cfg(target_os = "linux")]
    {
        if let Ok(file) = File::create("/dev/null") {
            while let Ok(bytes) = splice(
                output.as_fd(),
                None,
                file.as_fd(),
                None,
                CHUNK_SIZE,
                SpliceFFlags::empty(),
            ) {
                if bytes == 0 {
                    break;
                }
            }
        }
    }

    let mut output = output;
    let mut buf = [0; CHUNK_SIZE];
    while let Ok(bytes) = output.read(&mut buf) {
        if bytes == 0 {
            break;
        }
    }
}

/// Execute the given command and return a timing summary
pub fn execute_and_measure(mut command: Command) -> Result<TimerResult> {
    #[cfg(not(windows))]
    let cpu_timer = self::unix_timer::CPUTimer::start();

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_SUSPENDED);
    }

    let wallclock_timer = WallClockTimer::start();
    let mut child = command.spawn()?; // <-- Start process

    #[cfg(windows)]
    let cpu_timer = {
        unsafe { self::windows_timer::CPUTimer::start_suspended_process(&child) }
    };

    if let Some(output) = child.stdout.take() {
        discard(output);
    }

    let status = child.wait()?; // <-- Wait for completion

    let time_real = wallclock_timer.stop();
    let (time_user, time_system, memory_usage_byte) = cpu_timer.stop();

    // 🔹 Collect extra stats on Unix
    #[cfg(unix)]
    let (voluntary_cs, involuntary_cs, io_read_ops, io_write_ops) = {
        let usage: RUsage = getrusage(UsageWho::RUSAGE_CHILDREN)?;
        (
            usage.ru_nvcsw() as u64,
            usage.ru_nivcsw() as u64,
            usage.ru_inblock() as u64,
            usage.ru_oublock() as u64,
        )
    };

    //  Collect extra stats on Windows
    #[cfg(windows)]
    let (voluntary_cs, involuntary_cs, io_read_ops, io_write_ops) = {
        use std::mem::MaybeUninit;
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::{
            Foundation::HANDLE,
            System::Threading::GetProcessIoCounters,
            System::Threading::IO_COUNTERS,
        };

        let handle: HANDLE = child.as_raw_handle() as HANDLE;
        let mut counters = MaybeUninit::<IO_COUNTERS>::uninit();

        let success = unsafe { GetProcessIoCounters(handle, counters.as_mut_ptr()) };
        if success == 0 {
            (0, 0, 0, 0)
        } else {
            let counters = unsafe { counters.assume_init() };
            (
                0, // Context switches not available via this API
                0,
                counters.ReadOperationCount as u64,
                counters.WriteOperationCount as u64,
            )
        }
    };

    Ok(TimerResult {
        time_real,
        time_user,
        time_system,
        memory_usage_byte,
        voluntary_cs,
        involuntary_cs,
        io_read_ops,
        io_write_ops,
        status,
    })
}

