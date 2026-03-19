use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadDirectoryChangesW, FILE_ACTION_ADDED, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_FLAG_OVERLAPPED, FILE_LIST_DIRECTORY, FILE_NOTIFY_CHANGE_FILE_NAME,
    FILE_NOTIFY_INFORMATION, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};
use windows::Win32::System::IO::{GetOverlappedResult, OVERLAPPED};

/// Watches a directory for file creation events using ReadDirectoryChangesW.
/// Returns the first created file name in a fixed-size buffer, or None on timeout.
/// No heap allocation is performed.
pub fn wait_for_create(dir: &[u16], timeout_ms: u32) -> Result<Option<([u16; 260], usize)>> {
    // Open directory handle with required flags
    let dir_handle = unsafe {
        CreateFileW(
            PCWSTR(dir.as_ptr()),
            FILE_LIST_DIRECTORY.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED,
            None,
        )?
    };

    // Create event for overlapped I/O
    let event = unsafe { CreateEventW(None, true, false, None)? };
    let mut overlapped = OVERLAPPED {
        hEvent: event,
        ..Default::default()
    };

    // Fixed buffer for directory change notifications
    // Size: enough for FILE_NOTIFY_INFORMATION + MAX_PATH filename (260 * 2 bytes)
    let mut buffer = [0u8; 1024];
    let mut bytes_returned = 0u32;

    // Start async directory monitoring
    unsafe {
        ReadDirectoryChangesW(
            dir_handle,
            buffer.as_mut_ptr() as _,
            buffer.len() as u32,
            false,                        // Don't watch subtree
            FILE_NOTIFY_CHANGE_FILE_NAME, // Watch for file name changes
            Some(&mut bytes_returned),
            Some(&mut overlapped),
            None,
        )?;
    }

    // Wait for event or timeout
    let wait_result = unsafe { WaitForSingleObject(event, timeout_ms) };
    match wait_result {
        WAIT_OBJECT_0 => {
            // Get the result of the overlapped operation
            unsafe {
                GetOverlappedResult(dir_handle, &overlapped, &mut bytes_returned, false)?;
            }

            // Process the notification buffer
            if bytes_returned == 0 {
                return Ok(None);
            }

            let mut offset = 0;
            while offset < buffer.len() {
                // Safety: We're reading from a buffer filled by ReadDirectoryChangesW
                let notify =
                    unsafe { &*(buffer.as_ptr().add(offset) as *const FILE_NOTIFY_INFORMATION) };

                if notify.Action == FILE_ACTION_ADDED {
                    // Extract filename into fixed-size buffer
                    let mut filename = [0u16; 260];
                    let name_len = (notify.FileNameLength as usize) / 2; // Convert bytes to u16 count
                    let copy_len = name_len.min(259); // Leave room for null terminator

                    // Copy filename from notification structure using slice copy
                    filename[..copy_len].copy_from_slice(&notify.FileName[..copy_len]);
                    filename[copy_len] = 0; // Null terminate

                    return Ok(Some((filename, copy_len)));
                }

                // Move to next entry if present
                if notify.NextEntryOffset == 0 {
                    break;
                }
                offset += notify.NextEntryOffset as usize;
            }

            Ok(None)
        }
        WAIT_TIMEOUT => Ok(None),
        _ => Err(Error::from_hresult(HRESULT::from_win32(
            unsafe { windows::Win32::Foundation::GetLastError() }.0,
        ))),
    }
}
