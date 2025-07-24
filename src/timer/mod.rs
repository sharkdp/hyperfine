mod wall_clock_timer;

#[cfg(windows)]
mod windows_timer;

#[cfg(not(windows))]
mod unix_timer;

#[cfg(target_os = "linux")]
use nix::fcntl::{splice, SpliceFFlags};
#[cfg(not(windows))]
use std::convert::TryFrom;
#[cfg(not(windows))]
use std::convert::TryInto;
#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(target_os = "linux")]
use std::os::fd::AsFd;

#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::CREATE_SUSPENDED;

use crate::benchmark::timing_result::TimingResult;
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

    /// Number of voluntary context switches
    pub voluntary_context_switches: u64,

    /// Number of involuntary context switches
    pub context_switches: u64,

    /// Number of times the filesystem had to perform input.
    pub filesystem_input: u64,

    /// Number of times the filesystem had to perform output.
    pub filesystem_output: u64,

    /// Number of minor page faults
    pub minor_page_faults: u64,

    /// Number of major page faults
    pub major_page_faults: u64,
}

/// Used to indicate the result of running a command
#[derive(Debug, Copy, Clone)]
pub struct TimerResult {
    pub timing: TimingResult,
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

        // Create the process in a suspended state so that we don't miss any cpu time between process creation and `CPUTimer` start.
        command.creation_flags(CREATE_SUSPENDED);
    }

    let wallclock_timer = WallClockTimer::start();
    let mut child = command.spawn()?;

    #[cfg(windows)]
    let cpu_timer = {
        // SAFETY: We created a suspended process
        unsafe { self::windows_timer::CPUTimer::start_suspended_process(&child) }
    };

    if let Some(output) = child.stdout.take() {
        // Handle CommandOutputPolicy::Pipe
        discard(output);
    }

    #[cfg(not(windows))]
    let (status, memory_usage_byte) = {
        use std::io::Error;
        use std::os::unix::process::ExitStatusExt;

        let mut raw_status = 0;
        let pid = child.id().try_into().expect("should convert to pid_t");
        // SAFETY: libc::rusage is Plain Old Data
        let mut rusage = unsafe { std::mem::zeroed() };
        // SAFETY: all syscall arguments are valid
        let result = unsafe { libc::wait4(pid, &raw mut raw_status, 0, &raw mut rusage) };
        if result != pid {
            return Err(Error::last_os_error().into());
        }

        // Linux and *BSD return the value in KibiBytes, Darwin flavors in bytes
        let max_rss_byte = if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
            rusage.ru_maxrss
        } else {
            rusage.ru_maxrss * 1024
        };
        let memory_usage_byte = u64::try_from(max_rss_byte).expect("should not be negative");

        let status = ExitStatus::from_raw(raw_status);

        (status, memory_usage_byte)
    };

    #[cfg(windows)]
    let (status, memory_usage_byte) = (child.wait()?, 0);

    let time_real = wallclock_timer.stop();

    #[cfg(not(windows))]
    let (
        time_user,
        time_system,
        voluntary_context_switches,
        context_switches,
        filesystem_input,
        filesystem_output,
        minor_page_faults,
        major_page_faults,
    ) = cpu_timer.stop();

    #[cfg(windows)]
    let (time_user, time_system, memory_usage_byte) = cpu_timer.stop();
    #[cfg(windows)]
    let (
        voluntary_context_switches,
        context_switches,
        filesystem_input,
        filesystem_output,
        minor_page_faults,
        major_page_faults,
    ) = (0, 0, 0, 0, 0, 0);

    let timing = TimingResult {
        time_real,
        time_user,
        time_system,
        memory_usage_byte,
        voluntary_context_switches,
        context_switches,
        filesystem_input,
        filesystem_output,
        minor_page_faults,
        major_page_faults,
    };

    Ok(TimerResult { timing, status })
}
