use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::sync::mpsc;
use std::thread;

use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, GENERIC_WRITE, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, WriteFile, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, OPEN_ALWAYS,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn create_file(path: &str) -> Result<HANDLE> {
    let wide_path = wide_null(OsStr::new(path));
    unsafe {
        CreateFileW(
            PCWSTR::from_raw(wide_path.as_ptr()),
            GENERIC_WRITE.0, // Convert GENERIC_ACCESS_RIGHTS to u32
            FILE_SHARE_READ,
            None,
            OPEN_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }
}

fn write_to_file(handle: HANDLE, data: &[u8]) -> Result<()> {
    let mut bytes_written = 0;
    unsafe { WriteFile(handle, Some(data), Some(&mut bytes_written), None) }
}

// Wrapper to make HANDLE Send-safe for thread transfer
struct SendHandle(HANDLE);
unsafe impl Send for SendHandle {}

fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();

    let handle = create_file("test.txt")?;
    let send_handle = SendHandle(handle);

    tx.send(send_handle).unwrap();

    let worker = thread::spawn(move || -> Result<()> {
        let SendHandle(handle) = rx.recv().unwrap();
        write_to_file(handle, b"Hello from thread!")?;
        unsafe { CloseHandle(handle)? };
        Ok(())
    });

    worker.join().unwrap()?;

    Ok(())
}
