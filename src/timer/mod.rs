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
use std::os::unix::process::ExitStatusExt;

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

fn discard_until(output: ChildStdout, ptn: &[u8]) -> Result<bool> {
    const CHUNK_SIZE: usize = 64 << 10;

    let mut output = output;
    let mut buf = [0; CHUNK_SIZE];

    let ptn_len = ptn.len();
    let lps = compute_lps_array(ptn);
    let mut j = 0; // position of the character in ptn
    let mut read_more = false;

    loop {
        let n = output.read(&mut buf)?;

        if n == 0 {
            return Ok(false);
        }

        let mut i = 0; // position of the character in buf
        if read_more && ptn[j] != buf[i] {
            if j != 0 {
                j = lps[j - 1];
            } else {
                i += 1;
            }
        }
        read_more = false;

        while i < n {
            if ptn[j] == buf[i] {
                i += 1;
                j += 1;
            }

            if j == ptn_len {
                return Ok(true);
            }

            if i == n {
                read_more = true;
                break;
            }

            if ptn[j] == buf[i] {
                continue;
            }

            if j != 0 {
                j = lps[j - 1];
            } else {
                i += 1;
            }
        }
    }
}

#[inline(always)]
fn compute_lps_array(pattern: &[u8]) -> Vec<usize> {
    let ptn_len = pattern.len();
    let mut lps = vec![0; ptn_len];

    // length of the previous longest prefix suffix
    let mut len = 0;
    lps[0] = 0;

    // the loop calculates lps[i] for i = 1 to ptn_len-1
    let mut i = 1;
    while i < ptn_len {
        if pattern[i] == pattern[len] {
            len += 1;
            lps[i] = len;
            i += 1;
        } else if len != 0 {
            len = lps[len - 1];
        } else {
            lps[i] = 0;
            i += 1;
        }
    }

    lps
}

/// Execute the given command and return a timing summary
pub fn execute_and_measure(mut command: Command, until: Option<&[u8]>) -> Result<TimerResult> {
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

    if let Some(ptn) = until {
        // Handle CommandOutputPolicy::Until
        let output = child
            .stdout
            .take()
            .expect("Expected a pipe when until text is present.");

        let status = if discard_until(output, ptn)? {
            ExitStatus::from_raw(0)
        } else {
            ExitStatus::from_raw(-1)
        };

        let time_real = wallclock_timer.stop();
        let (time_user, time_system, memory_usage_byte) = cpu_timer.stop();

        child.kill()?;
        child.wait()?;

        return Ok(TimerResult {
            time_real,
            time_user,
            time_system,
            memory_usage_byte,
            status,
        });
    }

    if let Some(output) = child.stdout.take() {
        // Handle CommandOutputPolicy::Pipe
        discard(output);
    }

    let status = child.wait()?;

    let time_real = wallclock_timer.stop();
    let (time_user, time_system, memory_usage_byte) = cpu_timer.stop();

    Ok(TimerResult {
        time_real,
        time_user,
        time_system,
        memory_usage_byte,
        status,
    })
}
