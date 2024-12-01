#![cfg(windows)]
#![warn(unsafe_op_in_unsafe_fn)]

use std::{mem, os::windows::io::AsRawHandle, process, ptr};

use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectBasicAccountingInformation,
        QueryInformationJobObject, JOBOBJECT_BASIC_ACCOUNTING_INFORMATION,
    },
};

#[cfg(feature = "windows_process_extensions_main_thread_handle")]
use std::os::windows::process::ChildExt;
#[cfg(feature = "windows_process_extensions_main_thread_handle")]
use windows_sys::Win32::System::Threading::ResumeThread;

#[cfg(not(feature = "windows_process_extensions_main_thread_handle"))]
use once_cell::sync::Lazy;
#[cfg(not(feature = "windows_process_extensions_main_thread_handle"))]
use windows_sys::{
    s, w,
    Win32::{
        Foundation::{NTSTATUS, STATUS_SUCCESS},
        System::LibraryLoader::{GetModuleHandleW, GetProcAddress},
    },
};

use crate::util::units::Second;

const HUNDRED_NS_PER_MS: i64 = 10;

#[cfg(not(feature = "windows_process_extensions_main_thread_handle"))]
#[allow(non_upper_case_globals)]
static NtResumeProcess: Lazy<unsafe extern "system" fn(ProcessHandle: HANDLE) -> NTSTATUS> =
    Lazy::new(|| {
        // SAFETY: Getting the module handle for ntdll.dll is safe
        let ntdll = unsafe { GetModuleHandleW(w!("ntdll.dll")) };
        assert!(ntdll != std::ptr::null_mut(), "GetModuleHandleW failed");

        // SAFETY: The ntdll handle is valid
        let nt_resume_process = unsafe { GetProcAddress(ntdll, s!("NtResumeProcess")) };

        // SAFETY: We transmute to the correct function signature
        unsafe { mem::transmute(nt_resume_process.unwrap()) }
    });

pub struct CPUTimer {
    job_object: HANDLE,
}

impl CPUTimer {
    pub unsafe fn start_suspended_process(child: &process::Child) -> Self {
        let child_handle = child.as_raw_handle() as HANDLE;

        // SAFETY: Creating a new job object is safe
        let job_object = unsafe { CreateJobObjectW(ptr::null_mut(), ptr::null_mut()) };
        assert!(
            job_object != std::ptr::null_mut(),
            "CreateJobObjectW failed"
        );

        // SAFETY: The job object handle is valid
        let ret = unsafe { AssignProcessToJobObject(job_object, child_handle) };
        assert!(ret != 0, "AssignProcessToJobObject failed");

        #[cfg(feature = "windows_process_extensions_main_thread_handle")]
        {
            // SAFETY: The main thread handle is valid
            let ret = unsafe { ResumeThread(child.main_thread_handle().as_raw_handle() as HANDLE) };
            assert!(ret != u32::MAX, "ResumeThread failed");
        }

        #[cfg(not(feature = "windows_process_extensions_main_thread_handle"))]
        {
            // Since we can't get the main thread handle on stable rust, we use
            // the undocumented but widely known `NtResumeProcess` function to
            // resume a process by it's handle.

            // SAFETY: The process handle is valid
            let ret = unsafe { NtResumeProcess(child_handle) };
            assert!(ret == STATUS_SUCCESS, "NtResumeProcess failed");
        }

        Self { job_object }
    }

    pub fn stop(&self) -> (Second, Second, u64) {
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

            // The `TotalUserTime` is "The total amount of user-mode execution time for
            // all active processes associated with the job, as well as all terminated processes no
            // longer associated with the job, in 100-nanosecond ticks."
            let user: i64 = job_object_info.TotalUserTime / HUNDRED_NS_PER_MS;

            // The `TotalKernelTime` is "The total amount of kernel-mode execution time
            // for all active processes associated with the job, as well as all terminated
            // processes no longer associated with the job, in 100-nanosecond ticks."
            let kernel: i64 = job_object_info.TotalKernelTime / HUNDRED_NS_PER_MS;
            (user as f64 * 1e-6, kernel as f64 * 1e-6, 0)
        } else {
            (0.0, 0.0, 0)
        }
    }
}

impl Drop for CPUTimer {
    fn drop(&mut self) {
        // SAFETY: A valid job object got created in `start_suspended_process`
        unsafe { CloseHandle(self.job_object) };
    }
}
