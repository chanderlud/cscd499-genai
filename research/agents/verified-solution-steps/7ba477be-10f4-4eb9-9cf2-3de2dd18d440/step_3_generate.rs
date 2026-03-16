use std::ffi::OsStr;
use std::io;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use windows::core::{Error, HRESULT, PCWSTR};
use windows::Win32::Foundation::{
    CloseHandle, ERROR_IO_PENDING, HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, WriteFile, FILE_FLAG_OVERLAPPED, FILE_GENERIC_READ, FILE_GENERIC_WRITE,
    FILE_SHARE_NONE, OPEN_EXISTING, PIPE_ACCESS_DUPLEX,
};
use windows::Win32::System::IO::{CancelIo, GetOverlappedResult, OVERLAPPED};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_MESSAGE,
    PIPE_TYPE_MESSAGE, PIPE_WAIT,
};
use windows::Win32::System::Threading::{CreateEventW, ResetEvent, WaitForSingleObject};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn create_pipe_name(name: &str) -> Vec<u16> {
    let full_name = format!(r"\\.\pipe\{}", name);
    wide_null(OsStr::new(&full_name))
}

fn server_thread(pipe_name: Vec<u16>, msg: String, result_tx: mpsc::Sender<io::Result<String>>) {
    let result = (|| -> io::Result<String> {
        // Create the named pipe with message mode and overlapped I/O
        let pipe_handle = unsafe {
            CreateNamedPipeW(
                PCWSTR(pipe_name.as_ptr()),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
                1,
                4096,
                4096,
                0,
                None,
            )
        };

        if pipe_handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }

        // Create event for overlapped operations
        let event = unsafe { CreateEventW(None, true, false, None) }?;
        if event.is_invalid() {
            unsafe { CloseHandle(pipe_handle).ok() };
            return Err(io::Error::last_os_error());
        }

        // Connect to the pipe (wait for client)
        let mut overlapped = OVERLAPPED {
            hEvent: event,
            ..Default::default()
        };

        // Reset event before operation
        unsafe { ResetEvent(event) }?;

        let connect_result = unsafe { ConnectNamedPipe(pipe_handle, Some(&mut overlapped)) };
        if let Err(e) = connect_result {
            if e.code() != HRESULT::from_win32(ERROR_IO_PENDING.0) {
                unsafe {
                    CloseHandle(event).ok();
                    CloseHandle(pipe_handle).ok();
                }
                return Err(e.into());
            }

            // Wait for connection with timeout
            let wait_result = unsafe { WaitForSingleObject(event, 5000) };
            if wait_result != WAIT_OBJECT_0 {
                unsafe {
                    CancelIo(pipe_handle).ok();
                    CloseHandle(event).ok();
                    CloseHandle(pipe_handle).ok();
                }
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Timeout waiting for client connection",
                ));
            }
        }

        // Read message from client
        let mut buffer = [0u8; 4096];
        let mut bytes_read = 0u32;

        // Reset event before operation
        unsafe { ResetEvent(event) }?;

        let read_result = unsafe {
            ReadFile(
                pipe_handle,
                Some(&mut buffer),
                Some(&mut bytes_read),
                Some(&mut overlapped),
            )
        };

        if let Err(e) = read_result {
            if e.code() != HRESULT::from_win32(ERROR_IO_PENDING.0) {
                unsafe {
                    DisconnectNamedPipe(pipe_handle).ok();
                    CloseHandle(event).ok();
                    CloseHandle(pipe_handle).ok();
                }
                return Err(e.into());
            }

            // Wait for read with timeout
            let wait_result = unsafe { WaitForSingleObject(event, 5000) };
            if wait_result != WAIT_OBJECT_0 {
                unsafe {
                    CancelIo(pipe_handle).ok();
                    DisconnectNamedPipe(pipe_handle).ok();
                    CloseHandle(event).ok();
                    CloseHandle(pipe_handle).ok();
                }
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Timeout waiting for client message",
                ));
            }

            // Get the number of bytes read
            let mut bytes_transferred = 0u32;
            unsafe {
                GetOverlappedResult(
                    pipe_handle,
                    &overlapped,
                    &mut bytes_transferred,
                    false,
                )
            }?;
            bytes_read = bytes_transferred;
        }

        // Convert to string and uppercase
        let received = String::from_utf8_lossy(&buffer[..bytes_read as usize]);
        let uppercased: String = received
            .chars()
            .map(|c| {
                if c.is_ascii_lowercase() {
                    (c as u8 - b'a' + b'A') as char
                } else {
                    c
                }
            })
            .collect();

        // Write response back to client
        let response = uppercased.as_bytes();
        let mut bytes_written = 0u32;

        // Reset event before operation
        unsafe { ResetEvent(event) }?;

        let write_result = unsafe {
            WriteFile(
                pipe_handle,
                Some(response),
                Some(&mut bytes_written),
                Some(&mut overlapped),
            )
        };

        if let Err(e) = write_result {
            if e.code() != HRESULT::from_win32(ERROR_IO_PENDING.0) {
                unsafe {
                    DisconnectNamedPipe(pipe_handle).ok();
                    CloseHandle(event).ok();
                    CloseHandle(pipe_handle).ok();
                }
                return Err(e.into());
            }

            // Wait for write with timeout
            let wait_result = unsafe { WaitForSingleObject(event, 5000) };
            if wait_result != WAIT_OBJECT_0 {
                unsafe {
                    CancelIo(pipe_handle).ok();
                    DisconnectNamedPipe(pipe_handle).ok();
                    CloseHandle(event).ok();
                    CloseHandle(pipe_handle).ok();
                }
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Timeout waiting to write response",
                ));
            }
        }

        // Cleanup
        unsafe {
            DisconnectNamedPipe(pipe_handle).ok();
            CloseHandle(event).ok();
            CloseHandle(pipe_handle).ok();
        }

        Ok(uppercased)
    })();

    let _ = result_tx.send(result);
}

fn client_connect_and_send(pipe_name: &[u16], msg: &str) -> io::Result<String> {
    // Open the pipe
    let desired_access = FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0;
    let pipe_handle = unsafe {
        CreateFileW(
            PCWSTR(pipe_name.as_ptr()),
            desired_access,
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_OVERLAPPED,
            None,
        )
    };

    if pipe_handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }

    // Create event for overlapped operations
    let event = unsafe { CreateEventW(None, true, false, None) }?;
    if event.is_invalid() {
        unsafe { CloseHandle(pipe_handle).ok() };
        return Err(io::Error::last_os_error());
    }

    let mut overlapped = OVERLAPPED {
        hEvent: event,
        ..Default::default()
    };

    // Send message to server
    let msg_bytes = msg.as_bytes();
    let mut bytes_written = 0u32;

    // Reset event before operation
    unsafe { ResetEvent(event) }?;

    let write_result = unsafe {
        WriteFile(
            pipe_handle,
            Some(msg_bytes),
            Some(&mut bytes_written),
            Some(&mut overlapped),
        )
    };

    if let Err(e) = write_result {
        if e.code() != HRESULT::from_win32(ERROR_IO_PENDING.0) {
            unsafe {
                CloseHandle(event).ok();
                CloseHandle(pipe_handle).ok();
            }
            return Err(e.into());
        }

        // Wait for write with timeout
        let wait_result = unsafe { WaitForSingleObject(event, 5000) };
        if wait_result != WAIT_OBJECT_0 {
            unsafe {
                CancelIo(pipe_handle).ok();
                CloseHandle(event).ok();
                CloseHandle(pipe_handle).ok();
            }
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Timeout waiting to send message",
            ));
        }
    }

    // Read response from server
    let mut buffer = [0u8; 4096];
    let mut bytes_read = 0u32;

    // Reset event before operation
    unsafe { ResetEvent(event) }?;

    let read_result = unsafe {
        ReadFile(
            pipe_handle,
            Some(&mut buffer),
            Some(&mut bytes_read),
            Some(&mut overlapped),
        )
    };

    if let Err(e) = read_result {
        if e.code() != HRESULT::from_win32(ERROR_IO_PENDING.0) {
            unsafe {
                CloseHandle(event).ok();
                CloseHandle(pipe_handle).ok();
            }
            return Err(e.into());
        }

        // Wait for read with timeout
        let wait_result = unsafe { WaitForSingleObject(event, 5000) };
        if wait_result != WAIT_OBJECT_0 {
            unsafe {
                CancelIo(pipe_handle).ok();
                CloseHandle(event).ok();
                CloseHandle(pipe_handle).ok();
            }
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Timeout waiting for server response",
            ));
        }

        // Get the number of bytes read
        let mut bytes_transferred = 0u32;
        unsafe {
            GetOverlappedResult(
                pipe_handle,
                &overlapped,
                &mut bytes_transferred,
                false,
            )
        }?;
        bytes_read = bytes_transferred;
    }

    // Cleanup
    unsafe {
        CloseHandle(event).ok();
        CloseHandle(pipe_handle).ok();
    }

    // Convert response to string
    let response = String::from_utf8_lossy(&buffer[..bytes_read as usize]);
    Ok(response.into_owned())
}

pub fn named_pipe_uppercase_echo(pipe_name: &str, msg: &str) -> io::Result<String> {
    let pipe_name_wide = create_pipe_name(pipe_name);
    let (result_tx, result_rx) = mpsc::channel();

    // Start server thread
    let server_handle = thread::spawn({
        let pipe_name_wide = pipe_name_wide.clone();
        let msg = msg.to_string();
        move || {
            server_thread(pipe_name_wide, msg, result_tx);
        }
    });

    // Give server a moment to start
    thread::sleep(Duration::from_millis(100));

    // Run client
    let client_result = client_connect_and_send(&pipe_name_wide, msg);

    // Wait for server thread with timeout
    let server_timeout = Duration::from_secs(5);
    match server_handle.join() {
        Ok(_) => {}
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Server thread panicked",
            ));
        }
    }

    // Get server result
    match result_rx.recv_timeout(server_timeout) {
        Ok(server_result) => {
            // Check both client and server results
            let client_response = client_result?;
            let server_response = server_result?;
            
            // Verify they match (should be the same uppercase string)
            if client_response != server_response {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Client and server responses don't match",
                ));
            }
            
            Ok(client_response)
        }
        Err(mpsc::RecvTimeoutError::Timeout) => Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "Timeout waiting for server result",
        )),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "Server thread disconnected",
        )),
    }
}