mod wall_clock_timer;

#[cfg(windows)]
mod windows_timer;

#[cfg(all(unix, not(target_os = "linux")))]
mod unix_timer;

use crate::util::units::Second;
use wall_clock_timer::WallClockTimer;

use std::io::Read;
use std::process::{ChildStdout, Command, ExitStatus};

use anyhow::Result;

#[cfg(all(unix, not(target_os = "linux")))]
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
    /// Wall-clock time of the command execution (post-exec on Linux)
    pub time_real: Second,
    /// Full wall-clock time including fork overhead (for scheduling purposes)
    pub time_real_full: Second,
    pub time_user: Second,
    pub time_system: Second,
    pub memory_usage_byte: u64,
    /// The exit status of the process
    pub status: ExitStatus,
}

/// Discard the output of a child process.
#[cfg(target_os = "linux")]
fn discard(output: ChildStdout) {
    use nix::fcntl::{splice, SpliceFFlags};
    use std::fs::File;
    use std::os::fd::AsFd;

    const CHUNK_SIZE: usize = 64 << 10;

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

    let mut output = output;
    let mut buf = [0; CHUNK_SIZE];
    while let Ok(bytes) = output.read(&mut buf) {
        if bytes == 0 {
            break;
        }
    }
}

/// Discard the output of a child process.
#[cfg(not(target_os = "linux"))]
fn discard(output: ChildStdout) {
    const CHUNK_SIZE: usize = 64 << 10;

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

// =============================================================================
// Windows implementation
// =============================================================================

#[cfg(windows)]
pub fn execute_and_measure(mut command: Command, until: Option<&[u8]>) -> Result<TimerResult> {
    use std::os::windows::process::CommandExt;
    use std::os::windows::process::ExitStatusExt;
    use windows_sys::Win32::System::Threading::CREATE_SUSPENDED;

    // Create the process in a suspended state so that we don't miss any cpu time
    // between process creation and `CPUTimer` start.
    command.creation_flags(CREATE_SUSPENDED);

    let wallclock_timer = WallClockTimer::start();
    let mut child = command.spawn()?;

    // SAFETY: We created a suspended process
    let cpu_timer = unsafe { self::windows_timer::CPUTimer::start_suspended_process(&child) };

    if let Some(ptn) = until {
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
            time_real_full: time_real,
            time_user,
            time_system,
            memory_usage_byte,
            status,
        });
    }

    if let Some(output) = child.stdout.take() {
        discard(output);
    }

    let status = child.wait()?;

    let time_real = wallclock_timer.stop();
    let (time_user, time_system, memory_usage_byte) = cpu_timer.stop();

    Ok(TimerResult {
        time_real,
        time_real_full: time_real,
        time_user,
        time_system,
        memory_usage_byte,
        status,
    })
}

// =============================================================================
// Unix (non-Linux) implementation - macOS, BSD, etc.
// =============================================================================

#[cfg(all(unix, not(target_os = "linux")))]
pub fn execute_and_measure(mut command: Command, until: Option<&[u8]>) -> Result<TimerResult> {
    use std::os::unix::process::ExitStatusExt;

    let cpu_timer = self::unix_timer::CPUTimer::start();
    let wallclock_timer = WallClockTimer::start();
    let mut child = command.spawn()?;

    if let Some(ptn) = until {
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

        // child.kill() sends SIGKILL, we don't really want that.
        use nix::sys::signal::{self, Signal};
        use nix::unistd::Pid;
        signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM)?;

        child.wait()?;

        return Ok(TimerResult {
            time_real,
            time_real_full: time_real,
            time_user,
            time_system,
            memory_usage_byte,
            status,
        });
    }

    if let Some(output) = child.stdout.take() {
        discard(output);
    }

    let status = child.wait()?;

    let time_real = wallclock_timer.stop();
    let (time_user, time_system, memory_usage_byte) = cpu_timer.stop();

    Ok(TimerResult {
        time_real,
        time_real_full: time_real,
        time_user,
        time_system,
        memory_usage_byte,
        status,
    })
}

// =============================================================================
// Linux implementation - with post-fork timing
// =============================================================================

/// Wait for a specific child process and get its resource usage.
/// Unlike `getrusage(RUSAGE_CHILDREN)`, this returns stats for just this child.
#[cfg(target_os = "linux")]
fn wait4_child(pid: i32) -> Result<(ExitStatus, libc::rusage)> {
    use std::mem::MaybeUninit;
    use std::os::unix::process::ExitStatusExt;

    let mut status: i32 = 0;
    let mut rusage = MaybeUninit::<libc::rusage>::uninit();

    let result = unsafe { libc::wait4(pid, &mut status, 0, rusage.as_mut_ptr()) };

    if result == -1 {
        return Err(std::io::Error::last_os_error().into());
    }

    let rusage = unsafe { rusage.assume_init() };
    let exit_status = ExitStatus::from_raw(status);

    Ok((exit_status, rusage))
}

/// Extract timing and memory info from rusage.
#[cfg(target_os = "linux")]
fn extract_rusage_stats(rusage: &libc::rusage) -> (Second, Second, u64) {
    const MICROSEC_PER_SEC: i64 = 1_000_000;

    let time_user = (rusage.ru_utime.tv_sec as i64 * MICROSEC_PER_SEC
        + rusage.ru_utime.tv_usec as i64) as f64
        * 1e-6;

    let time_system = (rusage.ru_stime.tv_sec as i64 * MICROSEC_PER_SEC
        + rusage.ru_stime.tv_usec as i64) as f64
        * 1e-6;

    // Linux returns ru_maxrss in kilobytes
    let memory_usage_byte = (rusage.ru_maxrss as u64) * 1024;

    (time_user, time_system, memory_usage_byte)
}

#[cfg(target_os = "linux")]
pub fn execute_and_measure(mut command: Command, until: Option<&[u8]>) -> Result<TimerResult> {
    use std::os::fd::AsRawFd;
    use std::os::unix::process::CommandExt;
    use std::os::unix::process::ExitStatusExt;

    // Create pipe for child to signal "ready to exec"
    // O_CLOEXEC ensures the pipe is closed on exec, which signals the parent
    let (pipe_read, pipe_write) = nix::unistd::pipe2(nix::fcntl::OFlag::O_CLOEXEC)?;
    let pipe_read_fd = pipe_read.as_raw_fd();
    let pipe_write_fd = pipe_write.as_raw_fd();

    // Set up pre_exec hook to signal readiness (pipe closes on exec due to CLOEXEC)
    // We need to keep the write end open until exec, then CLOEXEC closes it
    unsafe {
        command.pre_exec(move || {
            // The pipe_write fd will be closed automatically by CLOEXEC when exec happens
            // We don't need to do anything here - just existing is enough
            // But we need to reference pipe_write_fd to ensure the fd is inherited
            let _ = pipe_write_fd;
            Ok(())
        });
    }

    // Start FULL wall clock timer (includes fork overhead, for scheduling)
    let wallclock_timer_full = WallClockTimer::start();

    // Spawn the child
    let mut child = command.spawn()?;
    let pid = child.id() as i32;

    // Close our copy of the write end (drop the OwnedFd)
    drop(pipe_write);

    // Wait for child to reach exec (pipe closes, read returns 0)
    let mut buf = [0u8; 1];
    let _ = nix::unistd::read(pipe_read_fd, &mut buf); // Blocks until pipe closes (exec) or child dies
    drop(pipe_read);

    // Start POST-EXEC wall clock timer (excludes fork overhead, for reporting)
    let wallclock_timer = WallClockTimer::start();

    if let Some(ptn) = until {
        let output = child
            .stdout
            .take()
            .expect("Expected a pipe when until text is present.");

        let matched = discard_until(output, ptn)?;
        let time_real = wallclock_timer.stop();
        let time_real_full = wallclock_timer_full.stop();

        // child.kill() sends SIGKILL, we don't really want that.
        use nix::sys::signal::{self, Signal};
        use nix::unistd::Pid;
        signal::kill(Pid::from_raw(pid), Signal::SIGTERM)?;

        // Use wait4 to get per-child rusage
        let (_, rusage) = wait4_child(pid)?;
        let (time_user, time_system, memory_usage_byte) = extract_rusage_stats(&rusage);

        let status = if matched {
            ExitStatus::from_raw(0)
        } else {
            ExitStatus::from_raw(-1)
        };

        return Ok(TimerResult {
            time_real,
            time_real_full,
            time_user,
            time_system,
            memory_usage_byte,
            status,
        });
    }

    if let Some(output) = child.stdout.take() {
        discard(output);
    }

    // Use wait4 to get per-child rusage
    let (status, rusage) = wait4_child(pid)?;
    let time_real = wallclock_timer.stop();
    let time_real_full = wallclock_timer_full.stop();
    let (time_user, time_system, memory_usage_byte) = extract_rusage_stats(&rusage);

    Ok(TimerResult {
        time_real,
        time_real_full,
        time_user,
        time_system,
        memory_usage_byte,
        status,
    })
}
