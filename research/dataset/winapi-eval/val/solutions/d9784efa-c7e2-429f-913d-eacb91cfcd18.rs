use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use windows::core::{Error, Result, HRESULT, PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectAssociateCompletionPortInformation,
    SetInformationJobObject, JOBOBJECT_ASSOCIATE_COMPLETION_PORT,
};
use windows::Win32::System::Threading::{
    CreateProcessW, GetExitCodeProcess, ResumeThread, WaitForSingleObject, CREATE_SUSPENDED,
    PROCESS_INFORMATION, STARTUPINFOW,
};
use windows::Win32::System::IO::{CreateIoCompletionPort, GetQueuedCompletionStatus, OVERLAPPED};

const JOB_OBJECT_MSG_NEW_PROCESS: u32 = 1;
const JOB_OBJECT_MSG_EXIT_PROCESS: u32 = 2;
const JOB_OBJECT_MSG_ABNORMAL_EXIT_PROCESS: u32 = 3;
const JOB_OBJECT_MSG_ACTIVE_PROCESS_ZERO: u32 = 4;

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

pub fn run_in_job_collect_messages(command_line: &str, timeout_ms: u32) -> Result<(u32, Vec<u32>)> {
    let job = unsafe { CreateJobObjectW(None, None)? };
    if job.is_invalid() {
        return Err(Error::from_thread());
    }

    let iocp = unsafe { CreateIoCompletionPort(INVALID_HANDLE_VALUE, None, 0, 0)? };
    if iocp.is_invalid() {
        unsafe { CloseHandle(job)? };
        return Err(Error::from_thread());
    }

    let completion_key = 0x1234usize;
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

    let startup_info = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
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

    unsafe {
        AssignProcessToJobObject(job, process_info.hProcess)?;
    }

    unsafe {
        ResumeThread(process_info.hThread);
        CloseHandle(process_info.hThread)?;
    }

    let child_process = process_info.hProcess;
    let mut messages = Vec::new();
    let mut child_exit_code = 0u32;
    let mut child_exited = false;
    let mut active_processes = 1;

    let start_time = std::time::Instant::now();
    let timeout_duration = std::time::Duration::from_millis(timeout_ms as u64);

    while active_processes > 0 {
        let elapsed = start_time.elapsed();
        if elapsed >= timeout_duration {
            unsafe {
                CloseHandle(child_process)?;
                CloseHandle(job)?;
                CloseHandle(iocp)?;
            }
            return Err(Error::from_hresult(HRESULT::from_win32(0x000005B4)));
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
                if completion_key_out == completion_key {
                    let msg_id = bytes_transferred;
                    messages.push(msg_id);

                    match msg_id {
                        JOB_OBJECT_MSG_NEW_PROCESS => {
                            active_processes += 1;
                        }
                        JOB_OBJECT_MSG_EXIT_PROCESS | JOB_OBJECT_MSG_ABNORMAL_EXIT_PROCESS => {
                            active_processes -= 1;
                            let exited_pid = overlapped_out as u32;
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
                if e.code() == HRESULT::from_win32(0x00000102) {
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

    if !child_exited {
        unsafe {
            WaitForSingleObject(child_process, timeout_ms);
            GetExitCodeProcess(child_process, &mut child_exit_code)?;
        }
    }

    unsafe {
        CloseHandle(child_process)?;
        CloseHandle(job)?;
        CloseHandle(iocp)?;
    }

    Ok((child_exit_code, messages))
}
