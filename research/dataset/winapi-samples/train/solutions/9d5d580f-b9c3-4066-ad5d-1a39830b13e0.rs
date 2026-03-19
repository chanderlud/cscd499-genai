use std::path::Path;
use std::sync::mpsc;
use std::thread;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadDirectoryChangesW, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OVERLAPPED,
    FILE_LIST_DIRECTORY, FILE_NOTIFY_CHANGE_ATTRIBUTES, FILE_NOTIFY_CHANGE_CREATION,
    FILE_NOTIFY_CHANGE_DIR_NAME, FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_LAST_WRITE,
    FILE_NOTIFY_CHANGE_SECURITY, FILE_NOTIFY_CHANGE_SIZE, FILE_NOTIFY_INFORMATION,
    FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::IO::{GetOverlappedResult, OVERLAPPED};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn watch_dir_fixed_buffer<const N: usize>(
    path: &Path,
    recursive: bool,
    buffer: &mut [u8; N],
) -> Result<std::sync::mpsc::Receiver<Vec<u8>>> {
    // Validate path exists and is a directory
    if !path.exists() {
        return Err(Error::from_hresult(HRESULT::from_win32(2))); // ERROR_FILE_NOT_FOUND
    }
    if !path.is_dir() {
        return Err(Error::from_hresult(HRESULT::from_win32(5))); // ERROR_ACCESS_DENIED
    }

    let path_wide = wide_null(path.as_os_str());
    let (tx, rx) = mpsc::channel();

    // Open directory handle with overlapped I/O
    let dir_handle = unsafe {
        CreateFileW(
            PCWSTR(path_wide.as_ptr()),
            FILE_LIST_DIRECTORY.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED,
            None,
        )?
    };

    // Create event for overlapped I/O
    let event =
        unsafe { windows::Win32::System::Threading::CreateEventW(None, true, false, None)? };

    // Pass raw handle values as usize across thread boundary
    let dir_handle_raw = dir_handle.0 as usize;
    let event_raw = event.0 as usize;

    // Copy buffer contents to a Vec that can be moved into the thread
    let buffer_vec = buffer.to_vec();

    // Spawn worker thread
    let _ = thread::spawn(move || {
        // Reconstruct handles inside the thread from usize
        let dir_handle = HANDLE(dir_handle_raw as *mut _);
        let event = HANDLE(event_raw as *mut _);

        // Convert Vec back to array for watch_directory
        let mut thread_buffer = [0u8; N];
        thread_buffer.copy_from_slice(&buffer_vec);

        let result = watch_directory(dir_handle, event, recursive, &mut thread_buffer, tx);
        unsafe {
            let _ = CloseHandle(event);
            let _ = CloseHandle(dir_handle);
        }
        result
    });

    Ok(rx)
}

fn watch_directory<const N: usize>(
    dir_handle: HANDLE,
    event: HANDLE,
    recursive: bool,
    buffer: &mut [u8; N],
    tx: mpsc::Sender<Vec<u8>>,
) -> Result<()> {
    let mut overlapped = OVERLAPPED {
        hEvent: event,
        ..Default::default()
    };

    let filter = FILE_NOTIFY_CHANGE_FILE_NAME
        | FILE_NOTIFY_CHANGE_DIR_NAME
        | FILE_NOTIFY_CHANGE_ATTRIBUTES
        | FILE_NOTIFY_CHANGE_SIZE
        | FILE_NOTIFY_CHANGE_LAST_WRITE
        | FILE_NOTIFY_CHANGE_CREATION
        | FILE_NOTIFY_CHANGE_SECURITY;

    loop {
        // Reset event for new operation
        unsafe {
            windows::Win32::System::Threading::ResetEvent(event)?;
        }

        // Issue directory change notification
        unsafe {
            ReadDirectoryChangesW(
                dir_handle,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                recursive,
                filter,
                None,
                Some(&mut overlapped),
                None,
            )?;
        }

        // Wait for the operation to complete
        let wait_result = unsafe {
            windows::Win32::System::Threading::WaitForSingleObject(event, 0xFFFFFFFF)
            // INFINITE
        };

        if wait_result != WAIT_OBJECT_0 {
            return Err(Error::from_thread());
        }

        // Get the result of the overlapped operation
        let mut bytes_returned: u32 = 0;
        unsafe {
            GetOverlappedResult(dir_handle, &overlapped, &mut bytes_returned, false)?;
        }

        if bytes_returned == 0 {
            continue;
        }

        // Process the notification buffer
        let mut offset = 0;
        while offset < bytes_returned as usize {
            // Safety: We're reading from the buffer that was filled by ReadDirectoryChangesW
            let notify_info =
                unsafe { &*(buffer.as_ptr().add(offset) as *const FILE_NOTIFY_INFORMATION) };

            // Calculate the size of this record
            let record_size = std::mem::size_of::<FILE_NOTIFY_INFORMATION>()
                + (notify_info.FileNameLength as usize);

            // Copy the raw record bytes
            let record_bytes =
                unsafe { std::slice::from_raw_parts(buffer.as_ptr().add(offset), record_size) };

            // Send the raw record bytes through the channel
            if tx.send(record_bytes.to_vec()).is_err() {
                // Receiver dropped, stop watching
                return Ok(());
            }

            // Move to next record
            if notify_info.NextEntryOffset == 0 {
                break;
            }
            offset += notify_info.NextEntryOffset as usize;
        }
    }
}
