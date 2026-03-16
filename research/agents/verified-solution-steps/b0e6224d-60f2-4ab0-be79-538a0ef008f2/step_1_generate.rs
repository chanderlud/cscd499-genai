use windows::core::{Result, Error, HRESULT, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::System::JobObjects::{
    CreateJobObjectW, SetInformationJobObject, AssignProcessToJobObject,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOB_OBJECT_LIMIT_PROCESS_MEMORY, JOB_OBJECT_LIMIT_JOB_MEMORY,
    JOB_OBJECT_LIMIT_ACTIVE_PROCESS, JOB_OBJECT_LIMIT_JOB_TIME,
    JobObjectExtendedLimitInformation,
};
use windows::Win32::System::Threading::{
    CreateProcessW, WaitForSingleObject, ResumeThread,
    PROCESS_INFORMATION, STARTUPINFOW, CREATE_SUSPENDED,
};
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

pub fn run_in_job_with_timeout(command_line: &str, timeout_ms: u32) -> Result<bool> {
    // Create job object
    let job_handle = unsafe { CreateJobObjectW(None, None) }?;
    
    // Configure job limits
    let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    
    // Set time limit if specified
    if timeout_ms > 0 {
        limits.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_JOB_TIME;
        limits.BasicLimitInformation.PerJobUserTimeLimit = (timeout_ms as i64) * 10000; // Convert ms to 100ns units
    }
    
    // Apply job limits
    unsafe {
        SetInformationJobObject(
            job_handle,
            JobObjectExtendedLimitInformation,
            &limits as *const _ as *const _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )?;
    }
    
    // Prepare process creation
    let mut startup_info = STARTUPINFOW::default();
    startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    let mut process_info = PROCESS_INFORMATION::default();
    
    let command_line_wide = wide_null(OsStr::new(command_line));
    
    // Create process suspended
    unsafe {
        CreateProcessW(
            None,
            PCWSTR(command_line_wide.as_ptr()),
            None,
            None,
            false,
            CREATE_SUSPENDED,
            None,
            None,
            &mut startup_info,
            &mut process_info,
        )?;
    }
    
    // Assign process to job
    unsafe {
        AssignProcessToJobObject(job_handle, process_info.hProcess)?;
    }
    
    // Resume the process
    unsafe {
        ResumeThread(process_info.hThread);
    }
    
    // Wait for process or timeout
    let wait_result = unsafe {
        WaitForSingleObject(process_info.hProcess, timeout_ms)
    };
    
    // Clean up handles
    unsafe {
        let _ = CloseHandle(process_info.hThread);
        let _ = CloseHandle(process_info.hProcess);
        let _ = CloseHandle(job_handle);
    }
    
    match wait_result {
        WAIT_OBJECT_0 => Ok(false), // Process exited normally
        WAIT_TIMEOUT => Ok(true),   // Process was terminated by job timeout
        _ => Err(Error::from_hresult(HRESULT::from_win32(wait_result.0))),
    }
}