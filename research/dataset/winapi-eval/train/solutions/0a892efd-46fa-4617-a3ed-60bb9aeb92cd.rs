use std::io;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::thread;

use windows::Win32::Foundation::{ERROR_BROKEN_PIPE, HANDLE, WIN32_ERROR};
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile};
use windows::Win32::System::Pipes::CreatePipe;

pub fn pipe_checksum(data: &[u8]) -> io::Result<u32> {
    let (read_handle, write_handle): (OwnedHandle, OwnedHandle) = unsafe {
        let mut read = HANDLE::default();
        let mut write = HANDLE::default();

        CreatePipe(&mut read, &mut write, None, 0).map_err(io::Error::from)?;

        (
            OwnedHandle::from_raw_handle(read.0),
            OwnedHandle::from_raw_handle(write.0),
        )
    };

    let data = data.to_vec();

    let writer_thread = thread::spawn(move || -> io::Result<()> {
        let mut total_written = 0usize;

        while total_written < data.len() {
            let remaining = &data[total_written..];
            let mut written_this_call = 0u32;

            unsafe {
                WriteFile(
                    HANDLE(write_handle.as_raw_handle() as _),
                    Some(remaining),
                    Some(&mut written_this_call),
                    None,
                )
                    .map_err(io::Error::from)?;
            }

            total_written += written_this_call as usize;
        }

        Ok(())
    });

    let mut checksum = 0u32;
    let mut buffer = [0u8; 4096];

    loop {
        let mut bytes_read = 0u32;

        match unsafe {
            ReadFile(
                HANDLE(read_handle.as_raw_handle() as _),
                Some(&mut buffer),
                Some(&mut bytes_read),
                None,
            )
        } {
            Ok(()) => {
                // For anonymous pipes, closed writer is reported as ERROR_BROKEN_PIPE,
                // not as Ok with bytes_read == 0. This branch is still harmless.
                if bytes_read == 0 {
                    break;
                }

                for &byte in &buffer[..bytes_read as usize] {
                    checksum = checksum.wrapping_add(byte as u32);
                }
            }
            Err(e) if WIN32_ERROR::from_error(&e) == Some(ERROR_BROKEN_PIPE) => {
                break;
            }
            Err(e) => return Err(io::Error::from(e)),
        }
    }

    writer_thread
        .join()
        .map_err(|_| io::Error::other("writer thread panicked"))??;

    Ok(checksum)
}
