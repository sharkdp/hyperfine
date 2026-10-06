mod wall_clock_timer;

#[cfg(target_os = "linux")]
mod linux_counters;
#[cfg(target_os = "macos")]
mod macos_counters;

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

use crate::benchmark::measurement::Measurement;
use wall_clock_timer::WallClockTimer;

use std::io::Read;
use std::process::{ChildStdout, Command};

use anyhow::Result;

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
pub fn execute_and_measure(
    mut command: Command,
    _collect_hardware_counters: bool,
) -> Result<Measurement> {
    #[cfg(target_os = "linux")]
    let counters = _collect_hardware_counters.then(linux_counters::CounterGroup::new);
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

    #[cfg(target_os = "macos")]
    let (time_wall_clock, hardware_counters) =
        if _collect_hardware_counters && macos_counters::wait(&mut child).is_ok() {
            // Stop timing before querying counters, but leave the child available for wait4.
            (
                Some(wallclock_timer.stop()),
                macos_counters::read(child.id()),
            )
        } else {
            (None, Default::default())
        };
    let (time_user, time_system, memory_peak_resident, exit_status) = cpu_timer.stop(child)?;
    #[cfg(target_os = "macos")]
    let time_wall_clock = time_wall_clock.unwrap_or_else(|| wallclock_timer.stop());
    #[cfg(not(target_os = "macos"))]
    let time_wall_clock = wallclock_timer.stop();

    #[cfg(target_os = "linux")]
    let hardware_counters = counters
        .map(linux_counters::CounterGroup::read)
        .unwrap_or_default();
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    let hardware_counters = Default::default();

    Ok(Measurement {
        time_wall_clock,
        time_user,
        time_system,
        memory_peak_resident,
        hardware_counters,
        exit_status,
    })
}
