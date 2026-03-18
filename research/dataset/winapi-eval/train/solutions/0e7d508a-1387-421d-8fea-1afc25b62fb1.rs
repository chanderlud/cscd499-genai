use std::path::Path;
use std::sync::mpsc;
use std::thread;
use windows::Win32::Foundation::{
CloseHandle, ERROR_DIRECTORY, ERROR_IO_PENDING, ERROR_NOTIFY_ENUM_DIR, HANDLE, WAIT_OBJECT_0,
};
use windows::Win32::Storage::FileSystem::{
CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OVERLAPPED, FILE_LIST_DIRECTORY,
FILE_NOTIFY_CHANGE_ATTRIBUTES, FILE_NOTIFY_CHANGE_CREATION, FILE_NOTIFY_CHANGE_DIR_NAME,
FILE_NOTIFY_CHANGE_FILE_NAME, FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_NOTIFY_CHANGE_SIZE,
FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, ReadDirectoryChangesW,
};
use windows::Win32::System::IO::{GetOverlappedResult, OVERLAPPED};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};
use windows::core::{PCWSTR, Result};

#[derive(Clone, Copy)]
struct SendHandle(HANDLE);

unsafe impl Send for SendHandle {}
unsafe impl Sync for SendHandle {}

impl SendHandle {
fn into_inner(self) -> HANDLE {
self.0
}
}

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
use std::{iter::once, os::windows::ffi::OsStrExt};
s.encode_wide().chain(once(0)).collect()
}

pub fn watch_dir(path: &Path, recursive: bool) -> Result<std::sync::mpsc::Receiver<Vec<u8>>> {
if !path.is_dir() {
return Err(windows::core::Error::new(
ERROR_DIRECTORY.to_hresult(),
"path is not a directory",
));
}

    let wide_path = wide_null(path.as_os_str());

    let dir_handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_LIST_DIRECTORY.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED,
            None,
        )?
    };

    let event = unsafe { CreateEventW(None, false, false, PCWSTR::null())? };

    let dir_handle = SendHandle(dir_handle);
    let event = SendHandle(event);

    let (tx, rx) = mpsc::channel();

    let filter = FILE_NOTIFY_CHANGE_FILE_NAME
        | FILE_NOTIFY_CHANGE_DIR_NAME
        | FILE_NOTIFY_CHANGE_ATTRIBUTES
        | FILE_NOTIFY_CHANGE_SIZE
        | FILE_NOTIFY_CHANGE_LAST_WRITE
        | FILE_NOTIFY_CHANGE_CREATION;

    thread::spawn(move || {
        let dir_handle = dir_handle.into_inner();
        let event = event.into_inner();

        let mut buffer = vec![0u8; 4096];

        loop {
            let mut overlapped = OVERLAPPED::default();
            overlapped.hEvent = event;

            let result = unsafe {
                ReadDirectoryChangesW(
                    dir_handle,
                    buffer.as_mut_ptr() as *mut _,
                    buffer.len() as u32,
                    recursive,
                    filter,
                    None,
                    Some(&mut overlapped),
                    None,
                )
            };

            if let Err(e) = result {
                if e.code() != ERROR_IO_PENDING.to_hresult() {
                    break;
                }
            }

            let wait_result = unsafe { WaitForSingleObject(event, 0xFFFFFFFF) };
            if wait_result != WAIT_OBJECT_0 {
                break;
            }

            let mut bytes_transferred = 0u32;
            if let Err(e) = unsafe {
                GetOverlappedResult(
                    dir_handle,
                    &overlapped as *const _,
                    &mut bytes_transferred,
                    false,
                )
            } {
                if e.code() == ERROR_NOTIFY_ENUM_DIR.to_hresult() {
                    buffer.resize(buffer.len() * 2, 0);
                    continue;
                }
                break;
            }

            if bytes_transferred == 0 {
                continue;
            }

            let notification_data = buffer[..bytes_transferred as usize].to_vec();
            if tx.send(notification_data).is_err() {
                break;
            }
        }

        unsafe {
            let _ = CloseHandle(dir_handle);
            let _ = CloseHandle(event);
        }
    });

    Ok(rx)
}