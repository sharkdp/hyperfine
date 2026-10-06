use std::fs::File;
use std::io::Read;

use crate::benchmark::measurement::HardwareCounters;

/// Disabled events on the spawning thread, inherited and enabled by the child's exec.
/// The parent never enables its own events, so its work is not counted.
pub struct CounterGroup {
    events: [Option<File>; 5],
}

impl CounterGroup {
    #[cfg(all(
        target_pointer_width = "64",
        any(
            target_arch = "aarch64",
            target_arch = "x86_64",
            target_arch = "riscv64",
            target_arch = "powerpc64",
            target_arch = "loongarch64"
        )
    ))]
    pub fn new() -> Self {
        use perf_event_open_sys::{bindings, perf_event_open};
        use std::os::fd::{AsRawFd, FromRawFd};

        let mut group_leader = -1;
        let events = [
            bindings::PERF_COUNT_HW_CPU_CYCLES,
            bindings::PERF_COUNT_HW_INSTRUCTIONS,
            bindings::PERF_COUNT_HW_CACHE_REFERENCES,
            bindings::PERF_COUNT_HW_CACHE_MISSES,
            bindings::PERF_COUNT_HW_BRANCH_MISSES,
        ]
        .map(|config| {
            let mut attr = bindings::perf_event_attr {
                type_: bindings::PERF_TYPE_HARDWARE,
                size: std::mem::size_of::<bindings::perf_event_attr>() as u32,
                config: u64::from(config),
                ..Default::default()
            };
            // Don't start counting immediately.
            attr.set_disabled(1);
            // Start counting when the child executes the benchmark program.
            attr.set_enable_on_exec(1);
            // New child processes and threads inherit the counters.
            attr.set_inherit(1);
            // Exclude kernel execution.
            attr.set_exclude_kernel(1);
            // Exclude hypervisor execution.
            attr.set_exclude_hv(1);

            // Attach counters to the calling thread.
            const CURRENT_THREAD: libc::pid_t = 0;
            // Count on whichever CPU the task runs on.
            const ANY_CPU: libc::c_int = -1;

            // SAFETY: attr is initialized and sized for the kernel API.
            let fd = unsafe {
                perf_event_open(
                    &mut attr,
                    CURRENT_THREAD,
                    ANY_CPU,
                    group_leader,
                    bindings::PERF_FLAG_FD_CLOEXEC.into(),
                )
            };
            if fd < 0 {
                return None;
            }
            // SAFETY: perf_event_open returned a new, valid descriptor.
            let event = unsafe { File::from_raw_fd(fd) };
            if group_leader == -1 {
                group_leader = event.as_raw_fd();
            }
            Some(event)
        });
        Self { events }
    }

    #[cfg(not(all(
        target_pointer_width = "64",
        any(
            target_arch = "aarch64",
            target_arch = "x86_64",
            target_arch = "riscv64",
            target_arch = "powerpc64",
            target_arch = "loongarch64"
        )
    )))]
    pub fn new() -> Self {
        Self {
            events: Default::default(),
        }
    }

    /// Read raw counts after the child has exited. Like poop, do not scale for multiplexing.
    pub fn read(mut self) -> HardwareCounters {
        let [cpu_cycles, instructions, cache_references, cache_misses, branch_misses] =
            self.events.each_mut().map(|event| {
                let mut bytes = [0; 8];
                event.as_mut()?.read_exact(&mut bytes).ok()?;
                Some(u64::from_ne_bytes(bytes))
            });
        HardwareCounters {
            cpu_cycles,
            instructions,
            cache_references,
            cache_misses,
            branch_misses,
        }
    }
}
