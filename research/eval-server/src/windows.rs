use std::{io, mem::size_of, ptr::null_mut};

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, TerminateJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};

pub struct Job(HANDLE);

impl Job {
    pub fn new_kill_on_close() -> io::Result<Self> {
        unsafe {
            let h = CreateJobObjectW(null_mut(), null_mut());
            if h == 0 {
                return Err(io::Error::last_os_error());
            }

            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            let ok = SetInformationJobObject(
                h,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );

            if ok == 0 {
                let e = io::Error::last_os_error();
                CloseHandle(h);
                return Err(e);
            }

            Ok(Job(h))
        }
    }

    pub fn assign(&self, process: HANDLE) -> io::Result<()> {
        unsafe {
            if AssignProcessToJobObject(self.0, process) == 0 {
                // Helpful error in logs if you want it
                let _code = GetLastError();
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }
    }

    pub fn terminate(&self) {
        unsafe {
            let _ = TerminateJobObject(self.0, 1);
        }
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            if self.0 != 0 {
                // Kill-on-close does the work. CloseHandle triggers termination.
                let _ = CloseHandle(self.0);
            }
        }
    }
}
