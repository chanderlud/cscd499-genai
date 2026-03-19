use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use windows::Win32::System::Threading::{
    GetThreadTimes, OpenThread, TerminateThread, THREAD_QUERY_INFORMATION, THREAD_TERMINATE,
};

pub fn terminate_process_threads(process_id: u32) -> windows::core::Result<Vec<u32>> {
    // Create a snapshot of all threads in the system
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) }?;

    // Ensure we close the snapshot handle when done
    let _snapshot_guard = HandleGuard(snapshot);

    let mut thread_entry = THREADENTRY32 {
        dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
        ..Default::default()
    };

    // Get the first thread in the snapshot
    let mut threads = Vec::new();
    let mut has_threads = unsafe { Thread32First(snapshot, &mut thread_entry) }.is_ok();

    while has_threads {
        // Check if this thread belongs to our target process
        if thread_entry.th32OwnerProcessID == process_id {
            threads.push(thread_entry.th32ThreadID);
        }

        // Move to next thread
        has_threads = unsafe { Thread32Next(snapshot, &mut thread_entry) }.is_ok();
    }

    if threads.is_empty() {
        return Ok(Vec::new());
    }

    // Find the main thread by comparing creation times
    let mut main_thread_id = None;
    let mut earliest_time = None;

    for &thread_id in &threads {
        // Open thread with query information permission to get creation time
        let thread_handle = unsafe { OpenThread(THREAD_QUERY_INFORMATION, false, thread_id) }?;

        let _guard = HandleGuard(thread_handle);

        let mut creation_time = FILETIME::default();
        let mut exit_time = FILETIME::default();
        let mut kernel_time = FILETIME::default();
        let mut user_time = FILETIME::default();

        let success = unsafe {
            GetThreadTimes(
                thread_handle,
                &mut creation_time,
                &mut exit_time,
                &mut kernel_time,
                &mut user_time,
            )
        };

        if success.is_ok() {
            let time =
                (creation_time.dwHighDateTime as u64) << 32 | creation_time.dwLowDateTime as u64;

            match earliest_time {
                None => {
                    earliest_time = Some(time);
                    main_thread_id = Some(thread_id);
                }
                Some(earliest) if time < earliest => {
                    earliest_time = Some(time);
                    main_thread_id = Some(thread_id);
                }
                _ => {}
            }
        }
    }

    // Terminate all threads except the main thread
    let mut terminated = Vec::new();

    for thread_id in threads {
        if Some(thread_id) == main_thread_id {
            continue;
        }

        // Open thread with terminate permission
        let thread_handle = unsafe { OpenThread(THREAD_TERMINATE, false, thread_id) };

        match thread_handle {
            Ok(handle) => {
                let _guard = HandleGuard(handle);

                // Terminate the thread (exit code 1)
                let result = unsafe { TerminateThread(handle, 1) };
                if result.is_ok() {
                    terminated.push(thread_id);
                }
            }
            Err(_) => {
                // Skip threads we can't open
                continue;
            }
        }
    }

    Ok(terminated)
}

// RAII guard to ensure handles are closed
struct HandleGuard(HANDLE);

impl Drop for HandleGuard {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}
