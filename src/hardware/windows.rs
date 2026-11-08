//! Hardware detection on Windows, based on the `windows-sys` bindings.

use super::{Cpu, Disk, Memory};

pub(super) fn cpu() -> Option<Cpu> {
    Some(Cpu {
        model: cpu_model(),
        physical_cores: physical_cores(),
        logical_cores: super::logical_cores(),
    })
}

pub(super) fn memory() -> Option<Memory> {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

    // SAFETY: MEMORYSTATUSEX is a plain data struct for which an all-zero value is
    // valid; dwLength is set immediately below, as the API requires.
    let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;

    // SAFETY: status is sized through dwLength and points to writable storage.
    let success = unsafe { GlobalMemoryStatusEx(&mut status) };
    (success != 0).then_some(Memory {
        total_bytes: status.ullTotalPhys,
    })
}

pub(super) fn disk() -> Option<Disk> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    let path = std::env::current_dir().ok()?;
    let wide_path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut available_bytes = 0u64;
    let mut total_bytes = 0u64;
    let mut total_free_bytes = 0u64;

    // SAFETY: the path is NUL-terminated and all three out-parameters are valid.
    let success = unsafe {
        GetDiskFreeSpaceExW(
            wide_path.as_ptr(),
            &mut available_bytes,
            &mut total_bytes,
            &mut total_free_bytes,
        )
    };
    if success == 0 {
        return None;
    }

    Some(Disk {
        path: path.to_string_lossy().into_owned(),
        total_bytes,
        available_bytes,
    })
}

/// Read `ProcessorNameString` for CPU 0 from the registry.
fn cpu_model() -> Option<String> {
    use windows_sys::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};

    let sub_key = wide_str(r"HARDWARE\DESCRIPTION\System\CentralProcessor\0");
    let value_name = wide_str("ProcessorNameString");
    let mut buffer = [0u16; 256];
    let mut size = std::mem::size_of_val(&buffer) as u32;

    // SAFETY: both strings are NUL-terminated and buffer is valid for `size` bytes.
    let status = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            sub_key.as_ptr(),
            value_name.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buffer.as_mut_ptr().cast(),
            &mut size,
        )
    };
    if status != 0 {
        return None;
    }

    let length = (size as usize / std::mem::size_of::<u16>()).min(buffer.len());
    let value = String::from_utf16_lossy(&buffer[..length]);
    Some(value.trim_end_matches('\0').trim().to_owned()).filter(|value| !value.is_empty())
}

/// Count physical cores. Hyperthreads are reported as one core by the API.
fn physical_cores() -> Option<usize> {
    use windows_sys::Win32::System::SystemInformation::{
        GetLogicalProcessorInformationEx, RelationProcessorCore,
    };

    let mut length = 0u32;
    // SAFETY: a null buffer asks the API for the required size.
    unsafe {
        GetLogicalProcessorInformationEx(RelationProcessorCore, std::ptr::null_mut(), &mut length)
    };
    if length == 0 {
        return None;
    }

    let mut buffer = vec![0u8; length as usize];
    // SAFETY: buffer is valid for writes of `length` bytes.
    let success = unsafe {
        GetLogicalProcessorInformationEx(
            RelationProcessorCore,
            buffer.as_mut_ptr().cast(),
            &mut length,
        )
    };
    if success == 0 {
        return None;
    }

    // Every entry starts with `Size: u32` followed by `Relationship: i32`, and we
    // only look at those two fields, so alignment does not matter.
    let mut cores = 0usize;
    let mut offset = 0usize;
    while offset + 8 <= length as usize {
        // SAFETY: the bounds check above guarantees both reads are in range.
        let size = unsafe { std::ptr::read_unaligned(buffer.as_ptr().add(offset).cast::<u32>()) };
        let relationship =
            unsafe { std::ptr::read_unaligned(buffer.as_ptr().add(offset + 4).cast::<i32>()) };

        if relationship == RelationProcessorCore {
            cores += 1;
        }
        if size == 0 {
            break;
        }
        offset += size as usize;
    }

    (cores > 0).then_some(cores)
}

/// Encode a string as a NUL-terminated UTF-16 buffer.
fn wide_str(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
