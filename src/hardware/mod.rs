//! Hardware properties of the machine that runs the benchmark.
//!
//! These are recorded in the `metadata` block of the JSON export, so that
//! results obtained on different machines can be told apart.
//!
//! Detection is best effort: every field is optional, and a property that we
//! cannot query on the current platform is omitted instead of being reported as
//! zero. Failing to detect hardware information never makes an export fail.

use serde::Serialize;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(windows)]
use windows as platform;

/// Fallback for platforms without a dedicated implementation.
#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
mod unsupported {
    use super::{Cpu, Disk, Memory};

    pub(super) fn cpu() -> Option<Cpu> {
        None
    }

    pub(super) fn memory() -> Option<Memory> {
        None
    }

    pub(super) fn disk() -> Option<Disk> {
        None
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
use unsupported as platform;

#[derive(Debug, Default, Serialize)]
pub struct Hardware {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<Cpu>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<Memory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk: Option<Disk>,
}

#[derive(Debug, Default, Serialize)]
pub struct Cpu {
    /// Human readable model name, for example `AMD Ryzen 9 7950X 16-Core Processor`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Number of physical cores, as reported by the operating system.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical_cores: Option<usize>,
    /// Number of logical cores available to this process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logical_cores: Option<usize>,
}

impl Cpu {
    fn is_empty(&self) -> bool {
        self.model.is_none() && self.physical_cores.is_none() && self.logical_cores.is_none()
    }
}

#[derive(Debug, Default, Serialize)]
pub struct Memory {
    /// Total amount of physical memory in bytes.
    pub total_bytes: u64,
}

#[derive(Debug, Default, Serialize)]
pub struct Disk {
    /// Absolute path of the filesystem that was inspected.
    pub path: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
}

impl Hardware {
    /// Detect the hardware properties of this machine.
    pub fn detect() -> Self {
        Self {
            cpu: platform::cpu().filter(|cpu| !cpu.is_empty()),
            memory: platform::memory(),
            disk: platform::disk(),
        }
    }
}

/// Number of logical cores that this process can use.
///
/// This respects CPU affinity masks and container quotas, which is what the
/// benchmark actually gets to run on.
fn logical_cores() -> Option<usize> {
    std::thread::available_parallelism()
        .ok()
        .map(std::num::NonZero::get)
}

/// Disk usage of the filesystem that backs the current working directory.
#[cfg(unix)]
#[allow(clippy::unnecessary_cast)] // `statvfs` counters are 32 bits wide on some targets.
fn disk_of_current_dir() -> Option<Disk> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;
    use std::os::unix::ffi::OsStrExt;

    let path = std::env::current_dir().ok()?;
    let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut stat = MaybeUninit::<libc::statvfs>::zeroed();
    // SAFETY: stat points to writable statvfs storage and c_path is a valid,
    // NUL-terminated path.
    let result = unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };
    if result != 0 {
        return None;
    }
    // SAFETY: the call succeeded and initialized the buffer.
    let stat = unsafe { stat.assume_init() };

    // The widths of the `statvfs` counters differ between targets, so widen them
    // explicitly before multiplying.
    let block_size = stat.f_frsize as u64;
    let total_bytes = (stat.f_blocks as u64).checked_mul(block_size)?;
    let available_bytes = (stat.f_bavail as u64).checked_mul(block_size)?;

    Some(Disk {
        path: path.to_string_lossy().into_owned(),
        total_bytes,
        available_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::{Cpu, Disk, Hardware, Memory};

    #[test]
    fn empty_properties_are_omitted() {
        assert_eq!(
            serde_json::to_value(Hardware::default()).unwrap(),
            serde_json::json!({})
        );
        assert_eq!(
            serde_json::to_value(Cpu::default()).unwrap(),
            serde_json::json!({})
        );
    }

    #[test]
    fn detected_properties_are_serialized() {
        let hardware = Hardware {
            cpu: Some(Cpu {
                model: Some("Example CPU".into()),
                physical_cores: Some(4),
                logical_cores: Some(8),
            }),
            memory: Some(Memory {
                total_bytes: 16_000_000_000,
            }),
            disk: Some(Disk {
                path: "/example".into(),
                total_bytes: 1000,
                available_bytes: 250,
            }),
        };

        assert_eq!(
            serde_json::to_value(hardware).unwrap(),
            serde_json::json!({
                "cpu": {
                    "model": "Example CPU",
                    "physical_cores": 4,
                    "logical_cores": 8
                },
                "memory": {"total_bytes": 16_000_000_000u64},
                "disk": {
                    "path": "/example",
                    "total_bytes": 1000,
                    "available_bytes": 250
                }
            })
        );
    }

    // The remaining tests read the real /proc filesystem, so they only run where
    // those files exist.
    #[cfg(target_os = "linux")]
    #[test]
    fn detects_this_machine() {
        let hardware = Hardware::detect();

        let cpu = hardware.cpu.expect("cpu information");
        assert!(
            cpu.logical_cores.unwrap_or_default() > 0,
            "expected at least one logical core"
        );
        // Virtual machines frequently do not name their CPU, so the model is only
        // checked for the absence of empty strings.
        assert!(cpu
            .model
            .as_ref()
            .is_none_or(|model| !model.trim().is_empty()));

        assert!(
            hardware.memory.expect("memory information").total_bytes > 0,
            "expected a non-zero amount of memory"
        );

        let disk = hardware.disk.expect("disk information");
        assert!(disk.total_bytes > 0, "expected a non-zero disk size");
        assert!(disk.available_bytes <= disk.total_bytes);
    }
}
