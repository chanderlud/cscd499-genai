use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, HRESULT, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_JOB_TIME,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows::Win32::System::Threading::{
    CreateProcessW, ResumeThread, WaitForSingleObject, CREATE_SUSPENDED, PROCESS_INFORMATION,
    STARTUPINFOW,
};

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
        limits.BasicLimitInformation.PerJobUserTimeLimit = (timeout_ms as i64) * 10000;
        // Convert ms to 100ns units
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

    let mut command_line_wide = wide_null(OsStr::new(command_line));
    let command_line_pwstr = PWSTR::from_raw(command_line_wide.as_mut_ptr());

    // Create process suspended
    unsafe {
        CreateProcessW(
            None,
            Some(command_line_pwstr),
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
    let wait_result = unsafe { WaitForSingleObject(process_info.hProcess, timeout_ms) };

    // Clean up handles
    unsafe {
        let _ = CloseHandle(process_info.hThread);
        let _ = CloseHandle(process_info.hProcess);
        let _ = CloseHandle(job_handle);
    }

    match wait_result {
        WAIT_OBJECT_0 => Ok(false), // Process exited normally
        WAIT_TIMEOUT => Ok(true),   // Process was terminated by job timeout
        WAIT_FAILED => Err(Error::from_thread()),
        _ => Err(Error::from_hresult(HRESULT::from_win32(wait_result.0))),
    }
}
