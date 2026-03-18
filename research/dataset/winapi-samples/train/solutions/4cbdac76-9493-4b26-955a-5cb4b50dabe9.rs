use std::ffi::OsStr;
use std::sync::mpsc;
use std::thread;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::GENERIC_READ;
use windows::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_MODE, OPEN_EXISTING,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        // Open the file in the thread.
        let file_path = OsStr::new("test.txt");
        let wide_path = wide_null(file_path);

        // SAFETY: We are calling a Windows API with a valid wide string.
        let file_handle = unsafe {
            CreateFileW(
                PCWSTR(wide_path.as_ptr()),
                GENERIC_READ.0,
                FILE_SHARE_MODE(0),
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            )
        };

        // Check if the handle is invalid.
        let file_handle = match file_handle {
            Ok(handle) => handle,
            Err(err) => {
                tx.send(Err(err)).unwrap();
                return;
            }
        };

        // Read the first 100 bytes.
        let mut buffer = [0u8; 100];
        let mut bytes_read = 0u32;
        // SAFETY: We are passing a valid handle and buffer.
        let read_result = unsafe {
            ReadFile(
                file_handle,
                Some(&mut buffer[..]),
                Some(&mut bytes_read),
                None,
            )
        };

        // Check for error in ReadFile.
        if let Err(err) = read_result {
            // Close the handle before returning.
            unsafe {
                let _ = CloseHandle(file_handle);
            };
            tx.send(Err(err)).unwrap();
            return;
        }

        // Close the handle.
        unsafe {
            let _ = CloseHandle(file_handle);
        };

        // Send the data we read.
        tx.send(Ok(buffer[..bytes_read as usize].to_vec())).unwrap();
    });

    // Wait for the thread to finish and get the result.
    let result = rx.recv().unwrap();

    match result {
        Ok(data) => {
            println!("Read {} bytes: {:?}", data.len(), data);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    handle.join().unwrap();

    Ok(())
}
