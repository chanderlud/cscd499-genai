use std::mem::size_of;
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{CloseHandle, ERROR_NO_MORE_FILES, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, Thread32First, Thread32Next,
    PROCESSENTRY32W, TH32CS_SNAPPROCESS, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use windows::Win32::System::Threading::{OpenThread, SuspendThread, THREAD_SUSPEND_RESUME};

fn find_process_id(process_name: &str) -> Result<u32> {
    // SAFETY: CreateToolhelp32Snapshot returns a valid handle on success
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }?;

    let mut process_entry = PROCESSENTRY32W::default();
    process_entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;

    // SAFETY: Process32FirstW initializes the struct and reads from the snapshot
    let mut found = unsafe { Process32FirstW(snapshot, &mut process_entry) }.is_ok();

    while found {
        // Convert the fixed-size wide char array to a string for comparison
        let exe_file = String::from_utf16_lossy(
            &process_entry.szExeFile[..process_entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(process_entry.szExeFile.len())],
        );

        if exe_file.eq_ignore_ascii_case(process_name) {
            // SAFETY: CloseHandle is safe to call with a valid handle
            unsafe { CloseHandle(snapshot)? };
            return Ok(process_entry.th32ProcessID);
        }

        // SAFETY: Process32NextW reads the next entry from the snapshot
        found = unsafe { Process32NextW(snapshot, &mut process_entry) }.is_ok();
    }

    // SAFETY: CloseHandle is safe to call with a valid handle
    unsafe { CloseHandle(snapshot)? };

    Err(Error::new(
        HRESULT::from_win32(ERROR_NO_MORE_FILES.0),
        format!("Process '{}' not found", process_name),
    ))
}

pub fn suspend_process_threads(process_name: &str) -> Result<Vec<HANDLE>> {
    let target_pid = find_process_id(process_name)?;

    // SAFETY: CreateToolhelp32Snapshot returns a valid handle on success
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) }?;

    let mut thread_entry = THREADENTRY32::default();
    thread_entry.dwSize = size_of::<THREADENTRY32>() as u32;

    let mut suspended_threads = Vec::new();

    // SAFETY: Thread32First initializes the struct and reads from the snapshot
    let mut found = unsafe { Thread32First(snapshot, &mut thread_entry) }.is_ok();

    while found {
        if thread_entry.th32OwnerProcessID == target_pid {
            // SAFETY: OpenThread returns a handle to the thread with requested access
            match unsafe { OpenThread(THREAD_SUSPEND_RESUME, false, thread_entry.th32ThreadID) } {
                Ok(thread_handle) => {
                    // SAFETY: SuspendThread operates on a valid thread handle
                    let result = unsafe { SuspendThread(thread_handle) };
                    if result != u32::MAX {
                        suspended_threads.push(thread_handle);
                    } else {
                        // Failed to suspend, close the handle immediately
                        // SAFETY: CloseHandle is safe to call with a valid handle
                        unsafe { CloseHandle(thread_handle)? };
                    }
                }
                Err(_) => {
                    // Thread may have exited or access denied, continue with next thread
                }
            }
        }

        // SAFETY: Thread32Next reads the next entry from the snapshot
        found = unsafe { Thread32Next(snapshot, &mut thread_entry) }.is_ok();
    }

    // SAFETY: CloseHandle is safe to call with a valid handle
    unsafe { CloseHandle(snapshot)? };

    Ok(suspended_threads)
}
