use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use windows::core::{Error, Result, HRESULT, PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectAssociateCompletionPortInformation,
    SetInformationJobObject, JOBOBJECT_ASSOCIATE_COMPLETION_PORT,
};
use windows::Win32::System::Threading::{
    CreateProcessW, GetExitCodeProcess, ResumeThread, WaitForSingleObject, CREATE_SUSPENDED,
    PROCESS_INFORMATION, STARTUPINFOW,
};
use windows::Win32::System::IO::{CreateIoCompletionPort, GetQueuedCompletionStatus, OVERLAPPED};

// Job object message constants from Windows API
const JOB_OBJECT_MSG_NEW_PROCESS: u32 = 1;
const JOB_OBJECT_MSG_EXIT_PROCESS: u32 = 2;
const JOB_OBJECT_MSG_ABNORMAL_EXIT_PROCESS: u32 = 3;
const JOB_OBJECT_MSG_ACTIVE_PROCESS_ZERO: u32 = 4;

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

pub fn run_in_job_collect_messages(command_line: &str, timeout_ms: u32) -> Result<(u32, Vec<u32>)> {
    // Create job object
    let job = unsafe { CreateJobObjectW(None, None)? };
    if job.is_invalid() {
        return Err(Error::from_thread());
    }

    // Create IO completion port
    let iocp = unsafe { CreateIoCompletionPort(INVALID_HANDLE_VALUE, None, 0, 0)? };
    if iocp.is_invalid() {
        unsafe { CloseHandle(job)? };
        return Err(Error::from_thread());
    }

    // Associate job with completion port
    let completion_key = 0x1234usize; // Arbitrary key for our job
    let assoc = JOBOBJECT_ASSOCIATE_COMPLETION_PORT {
        CompletionKey: completion_key as *mut _,
        CompletionPort: iocp,
    };
    let assoc_size = std::mem::size_of::<JOBOBJECT_ASSOCIATE_COMPLETION_PORT>() as u32;
    unsafe {
        SetInformationJobObject(
            job,
            JobObjectAssociateCompletionPortInformation,
            &assoc as *const _ as *const _,
            assoc_size,
        )?;
    }

    // Create process suspended
    let mut startup_info = STARTUPINFOW::default();
    startup_info.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    let mut process_info = PROCESS_INFORMATION::default();
    let mut cmd_line_w = wide_null(OsStr::new(command_line));
    let cmd_line_pwstr = PWSTR(cmd_line_w.as_mut_ptr());

    unsafe {
        CreateProcessW(
            PCWSTR::null(),
            Some(cmd_line_pwstr),
            None,
            None,
            false,
            CREATE_SUSPENDED,
            None,
            PCWSTR::null(),
            &startup_info,
            &mut process_info,
        )?;
    }

    // Assign process to job
    unsafe {
        AssignProcessToJobObject(job, process_info.hProcess)?;
    }

    // Resume the process
    unsafe {
        ResumeThread(process_info.hThread);
        CloseHandle(process_info.hThread)?;
    }

    let child_process = process_info.hProcess;
    let mut messages = Vec::new();
    let mut child_exit_code = 0u32;
    let mut child_exited = false;
    let mut active_processes = 1; // We started one process

    // Wait for messages until job becomes empty or timeout
    let start_time = std::time::Instant::now();
    let timeout_duration = std::time::Duration::from_millis(timeout_ms as u64);

    while active_processes > 0 {
        // Check timeout
        let elapsed = start_time.elapsed();
        if elapsed >= timeout_duration {
            unsafe {
                CloseHandle(child_process)?;
                CloseHandle(job)?;
                CloseHandle(iocp)?;
            }
            return Err(Error::from_hresult(HRESULT::from_win32(0x000005B4))); // ERROR_TIMEOUT
        }

        let remaining_ms = (timeout_duration - elapsed).as_millis() as u32;
        let mut completion_key_out = 0usize;
        let mut overlapped_out: *mut OVERLAPPED = null_mut();
        let mut bytes_transferred = 0u32;

        let result = unsafe {
            GetQueuedCompletionStatus(
                iocp,
                &mut bytes_transferred,
                &mut completion_key_out,
                &mut overlapped_out,
                remaining_ms,
            )
        };

        match result {
            Ok(()) => {
                // Check if this is our job's completion key
                if completion_key_out == completion_key {
                    let msg_id = bytes_transferred;
                    messages.push(msg_id);

                    match msg_id {
                        JOB_OBJECT_MSG_NEW_PROCESS => {
                            active_processes += 1;
                        }
                        JOB_OBJECT_MSG_EXIT_PROCESS | JOB_OBJECT_MSG_ABNORMAL_EXIT_PROCESS => {
                            active_processes -= 1;
                            // Check if this is our child process
                            let exited_pid = overlapped_out as u32; // Process ID is passed in overlapped
                            if exited_pid == process_info.dwProcessId {
                                child_exited = true;
                                unsafe {
                                    GetExitCodeProcess(child_process, &mut child_exit_code)?;
                                }
                            }
                        }
                        JOB_OBJECT_MSG_ACTIVE_PROCESS_ZERO => {
                            active_processes = 0;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                // Check if it's a timeout
                if e.code() == HRESULT::from_win32(0x00000102) {
                    // WAIT_TIMEOUT
                    continue;
                }
                unsafe {
                    CloseHandle(child_process)?;
                    CloseHandle(job)?;
                    CloseHandle(iocp)?;
                }
                return Err(e);
            }
        }
    }

    // If we haven't gotten the child exit code yet, wait for it
    if !child_exited {
        unsafe {
            WaitForSingleObject(child_process, timeout_ms);
            GetExitCodeProcess(child_process, &mut child_exit_code)?;
        }
    }

    // Clean up
    unsafe {
        CloseHandle(child_process)?;
        CloseHandle(job)?;
        CloseHandle(iocp)?;
    }

    Ok((child_exit_code, messages))
}