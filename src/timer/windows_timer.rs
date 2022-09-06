#![cfg(windows)]
#![warn(unsafe_op_in_unsafe_fn)]

use std::{mem, os::windows::io::AsRawHandle, process, ptr};

use winapi::{
    shared::{ntdef::NTSTATUS, ntstatus::STATUS_SUCCESS},
    um::{
        handleapi::CloseHandle,
        jobapi2::{AssignProcessToJobObject, CreateJobObjectW, QueryInformationJobObject},
        libloaderapi::{GetModuleHandleA, GetProcAddress},
        winnt::{
            JobObjectBasicAccountingInformation, HANDLE, JOBOBJECT_BASIC_ACCOUNTING_INFORMATION,
        },
    },
};

#[cfg(windows_process_extensions_main_thread_handle)]
use winapi::shared::minwindef::DWORD;

#[cfg(not(windows_process_extensions_main_thread_handle))]
use once_cell::sync::Lazy;

use crate::util::units::Second;

const HUNDRED_NS_PER_MS: i64 = 10;

#[cfg(not(windows_process_extensions_main_thread_handle))]
#[allow(non_upper_case_globals)]
static NtResumeProcess: Lazy<unsafe extern "system" fn(ProcessHandle: HANDLE) -> NTSTATUS> =
    Lazy::new(|| {
        // SAFETY: Getting the module handle for ntdll.dll is safe
        let ntdll = unsafe { GetModuleHandleA(b"ntdll.dll\0".as_ptr().cast()) };
        assert!(!ntdll.is_null(), "GetModuleHandleA failed");

        // SAFETY: The ntdll handle is valid
        let nt_resume_process =
            unsafe { GetProcAddress(ntdll, b"NtResumeProcess\0".as_ptr().cast()) };
        assert!(!nt_resume_process.is_null(), "GetProcAddress failed");

        // SAFETY: We transmute to the correct function signature
        unsafe { mem::transmute(nt_resume_process) }
    });

pub struct CPUTimer {
    job_object: HANDLE,
}

impl CPUTimer {
    pub unsafe fn start_suspended_process(child: &process::Child) -> Self {
        // SAFETY: Creating a new job object is safe
        let job_object = unsafe { CreateJobObjectW(ptr::null_mut(), ptr::null_mut()) };
        assert!(!job_object.is_null(), "CreateJobObjectW failed");

        // SAFETY: The job object handle is valid
        let ret = unsafe { AssignProcessToJobObject(job_object, child.as_raw_handle()) };
        assert!(ret != 0, "AssignProcessToJobObject failed");

        #[cfg(windows_process_extensions_main_thread_handle)]
        {
            // SAFETY: The main thread handle is valid
            let ret = unsafe { ResumeThread(child.main_thread_handle().as_raw_handle()) };
            assert!(ret != -1 as DWORD, "ResumeThread failed");
        }

        #[cfg(not(windows_process_extensions_main_thread_handle))]
        {
            // Since we can't get the main thread handle on stable rust, we use
            // the undocumented but widely known `NtResumeProcess` function to
            // resume a process by it's handle.

            // SAFETY: The process handle is valid
            let ret = unsafe { NtResumeProcess(child.as_raw_handle()) };
            assert!(ret == STATUS_SUCCESS, "NtResumeProcess failed");
        }

        Self { job_object }
    }

    pub fn stop(&self) -> (Second, Second) {
        let mut job_object_info =
            mem::MaybeUninit::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>::uninit();

        // SAFETY: A valid job object got created in `start_suspended_process`
        let res = unsafe {
            QueryInformationJobObject(
                self.job_object,
                JobObjectBasicAccountingInformation,
                job_object_info.as_mut_ptr().cast(),
                mem::size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                ptr::null_mut(),
            )
        };

        if res != 0 {
            // SAFETY: The job object info got correctly initialized
            let job_object_info = unsafe { job_object_info.assume_init() };

            // SAFETY: The `TotalUserTime` is "The total amount of user-mode execution time for
            // all active processes associated with the job, as well as all terminated processes no
            // longer associated with the job, in 100-nanosecond ticks." and is safe to extract
            let user: i64 = unsafe { job_object_info.TotalUserTime.QuadPart() } / HUNDRED_NS_PER_MS;

            // SAFETY: The `TotalKernelTime` is "The total amount of kernel-mode execution time
            // for all active processes associated with the job, as well as all terminated
            // processes no longer associated with the job, in 100-nanosecond ticks." and is safe
            // to extract
            let kernel: i64 =
                unsafe { job_object_info.TotalKernelTime.QuadPart() } / HUNDRED_NS_PER_MS;
            (user as f64 * 1e-6, kernel as f64 * 1e-6)
        } else {
            (0.0, 0.0)
        }
    }
}

impl Drop for CPUTimer {
    fn drop(self: &mut Self) {
        // SAFETY: A valid job object got created in `start_suspended_process`
        unsafe { CloseHandle(self.job_object) };
    }
}
