//! Hardware detection on macOS, based on `sysctl`.

use super::{Cpu, Disk, Memory};

use std::convert::TryFrom;
use std::ffi::CString;

pub(super) fn cpu() -> Option<Cpu> {
    Some(Cpu {
        model: sysctl_string("machdep.cpu.brand_string"),
        physical_cores: sysctl_u32("hw.physicalcpu").and_then(|cores| usize::try_from(cores).ok()),
        logical_cores: super::logical_cores(),
    })
}

pub(super) fn memory() -> Option<Memory> {
    Some(Memory {
        total_bytes: sysctl_u64("hw.memsize")?,
    })
}

pub(super) fn disk() -> Option<Disk> {
    super::disk_of_current_dir()
}

/// Read a `sysctl` value into `buffer`, returning the number of bytes written.
fn sysctl_raw(name: &str, buffer: &mut [u8]) -> Option<usize> {
    let c_name = CString::new(name).ok()?;
    let mut size = buffer.len();

    // SAFETY: buffer is valid for writes of `size` bytes and c_name is a valid,
    // NUL-terminated name.
    let status = unsafe {
        libc::sysctlbyname(
            c_name.as_ptr(),
            buffer.as_mut_ptr().cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };

    (status == 0).then_some(size)
}

fn sysctl_string(name: &str) -> Option<String> {
    // CPU brand strings are well below this size.
    let mut buffer = [0u8; 256];
    let size = sysctl_raw(name, &mut buffer)?;
    let mut value = buffer.get(..size)?;

    // The value is a NUL-terminated C string.
    while value.last() == Some(&0) {
        value = value.get(..value.len().checked_sub(1)?)?;
    }

    String::from_utf8(value.to_vec())
        .ok()
        .filter(|value| !value.is_empty())
}

fn sysctl_u32(name: &str) -> Option<u32> {
    let mut buffer = [0u8; std::mem::size_of::<u32>()];
    sysctl_raw(name, &mut buffer)?;
    Some(u32::from_ne_bytes(buffer))
}

fn sysctl_u64(name: &str) -> Option<u64> {
    let mut buffer = [0u8; std::mem::size_of::<u64>()];
    sysctl_raw(name, &mut buffer)?;
    Some(u64::from_ne_bytes(buffer))
}
