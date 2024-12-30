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

use crate::benchmark::quantity::{Byte, Second};
use wall_clock_timer::WallClockTimer;

use std::io::Read;
use std::process::{ChildStdout, Command, ExitStatus};

use anyhow::Result;

/// Used to indicate the result of running a command
#[derive(Debug, Copy, Clone)]
pub struct TimerResult {
    pub time_wall_clock: Second,
    pub time_user: Second,
    pub time_system: Second,
    pub memory_usage_byte: Byte,
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

    let (status, time_user, time_system, memory_usage_byte) = cpu_timer.stop(child)?;
    let time_wall_clock = wallclock_timer.stop();

    Ok(TimerResult {
        time_wall_clock,
        time_user,
        time_system,
        memory_usage_byte,
        status,
    })
}
