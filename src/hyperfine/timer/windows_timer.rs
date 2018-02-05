#![cfg(windows)]

use super::internal::CPUTimes;
use hyperfine::internal::Second;

use winapi::um::processthreadsapi::GetProcessTimes;
use winapi::um::winnt::HANDLE;

use std::mem;
use std::os::windows::io::RawHandle;

const HUNDRED_NS_PER_MS: i64 = 10;

pub trait WindowsTimer {
    type Result;

    fn start(handle: RawHandle) -> Self;
    fn stop(&self) -> Self::Result;
}

pub struct CPUTimer {
    handle: RawHandle,
}

impl WindowsTimer for CPUTimer {
    type Result = (Second, Second);

    fn start(handle: RawHandle) -> Self {
       CPUTimer { handle }
    }

    fn stop(&self) -> Self::Result {
        let times = get_cpu_times(self.handle);
        (times.user_usec as f64 * 1e-6, times.system_usec as f64 * 1e-6)
    }
}

/// Read CPU execution times (dummy for now)
fn get_cpu_times(handle: RawHandle) -> CPUTimes {
    let (user_usec, system_usec) = unsafe {
        let mut _ctime = mem::zeroed();
        let mut _etime = mem::zeroed();
        let mut kernel_time = mem::zeroed();
        let mut user_time = mem::zeroed();
        let res = GetProcessTimes(
            handle as HANDLE,
            &mut _ctime,
            &mut _etime,
            &mut kernel_time,
            &mut user_time,
        );

        // GetProcessTimes will exit with non-zero if success as per: https://msdn.microsoft.com/en-us/library/windows/desktop/ms683223(v=vs.85).aspx
        if res != 0 {
            // Extract times as laid out here: https://support.microsoft.com/en-us/help/188768/info-working-with-the-filetime-structure
            // Both user_time and kernel_time are spans that the proces spent in either. 
            let user: i64 = (((user_time.dwHighDateTime as i64) << 32) + user_time.dwLowDateTime as i64) / HUNDRED_NS_PER_MS;
            let kernel: i64 = (((kernel_time.dwHighDateTime as i64) << 32) + kernel_time.dwLowDateTime as i64) / HUNDRED_NS_PER_MS;
            (user, kernel)
        } else {
            (0, 0)
        }
    };

    CPUTimes {
        user_usec, system_usec,
    }
}
