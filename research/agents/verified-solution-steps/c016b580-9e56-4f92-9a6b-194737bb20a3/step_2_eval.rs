use windows::core::{Result, Error, PCWSTR, BOOL};
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE, CloseHandle, GetLastError, ERROR_PIPE_CONNECTED};
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile, FlushFileBuffers, CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_SHARE_MODE, FILE_CREATION_DISPOSITION, OPEN_EXISTING};
use windows::Win32::System::Pipes::{CreateNamedPipeW, ConnectNamedPipe, DisconnectNamedPipe, PIPE_ACCESS_DUPLEX, PIPE_TYPE_BYTE, PIPE_READMODE_BYTE, PIPE_WAIT, PIPE_UNLIMITED_INSTANCES, NMPWAIT_WAIT_FOREVER};
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE};

use std::io;
use std::mem;
use std::ptr;
use std::thread;
use std::sync::mpsc;

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn read_exact(handle: HANDLE, buf: &mut [u8]) -> io::Result<()> {
    let mut total_read = 0;
    while total_read < buf.len() {
        let mut bytes_read = 0u32;
        let remaining = &mut buf[total_read..];
        // SAFETY: We're calling ReadFile with valid parameters. The handle is valid,
        // the buffer is valid and we're passing the correct length.
        let result = unsafe {
            ReadFile(
                handle,
                Some(remaining),
                Some(&mut bytes_read),
                None,
            )
        };
        
        if let Err(e) = result {
            return Err(io::Error::from_raw_os_error(e.code().0));
        }
        
        if bytes_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Pipe closed unexpectedly",
            ));
        }
        total_read += bytes_read as usize;
    }
    Ok(())
}

fn write_exact(handle: HANDLE, buf: &[u8]) -> io::Result<()> {
    let mut total_written = 0;
    while total_written < buf.len() {
        let mut bytes_written = 0u32;
        let remaining = &buf[total_written..];
        // SAFETY: We're calling WriteFile with valid parameters. The handle is valid,
        // the buffer is valid and we're passing the correct length.
        let result = unsafe {
            WriteFile(
                handle,
                Some(remaining),
                Some(&mut bytes_written),
                None,
            )
        };
        
        if let Err(e) = result {
            return Err(io::Error::from_raw_os_error(e.code().0));
        }
        
        total_written += bytes_written as usize;
    }
    Ok(())
}

fn read_u32(handle: HANDLE) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    read_exact(handle, &mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn write_u32(handle: HANDLE, value: u32) -> io::Result<()> {
    write_exact(handle, &value.to_le_bytes())
}

fn server(pipe_name: String, n_frames: usize) -> io::Result<()> {
    let full_name = format!("\\\\.\\pipe\\{}", pipe_name);
    let wide_name = wide_null(std::ffi::OsStr::new(&full_name));
    
    // SAFETY: We're calling CreateNamedPipeW with valid parameters.
    let pipe_handle = unsafe {
        CreateNamedPipeW(
            PCWSTR(wide_name.as_ptr()),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
            PIPE_UNLIMITED_INSTANCES,
            4096,
            4096,
            0,
            None,
        )
    };
    
    if pipe_handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::from_raw_os_error(unsafe { GetLastError().0 as i32 }));
    }
    
    // SAFETY: We're calling ConnectNamedPipe with a valid handle.
    let connected = unsafe { ConnectNamedPipe(pipe_handle, None) };
    if let Err(e) = connected {
        let err = e.code().0 as u32;
        // ERROR_PIPE_CONNECTED means client already connected before ConnectNamedPipe
        if err != ERROR_PIPE_CONNECTED.0 {
            unsafe { let _ = CloseHandle(pipe_handle); };
            return Err(io::Error::from_raw_os_error(err as i32));
        }
    }
    
    // Read frames
    let mut frame_lengths = Vec::with_capacity(n_frames);
    for _ in 0..n_frames {
        let len = read_u32(pipe_handle)?;
        let mut frame_data = vec![0u8; len as usize];
        read_exact(pipe_handle, &mut frame_data)?;
        frame_lengths.push(len);
    }
    
    // Send response: count + lengths
    write_u32(pipe_handle, n_frames as u32)?;
    for &len in &frame_lengths {
        write_u32(pipe_handle, len)?;
    }
    
    // SAFETY: We're calling FlushFileBuffers and DisconnectNamedPipe with valid handles.
    unsafe {
        let _ = FlushFileBuffers(pipe_handle);
        let _ = DisconnectNamedPipe(pipe_handle);
        let _ = CloseHandle(pipe_handle);
    }
    
    Ok(())
}

fn client(pipe_name: String, frames: Vec<Vec<u8>>) -> io::Result<Vec<u32>> {
    let full_name = format!("\\\\.\\pipe\\{}", pipe_name);
    let wide_name = wide_null(std::ffi::OsStr::new(&full_name));
    
    // SAFETY: We're calling CreateFileW with valid parameters.
    let pipe_handle = unsafe {
        CreateFileW(
            PCWSTR(wide_name.as_ptr()),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_MODE(0),
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )
    };
    
    let pipe_handle = match pipe_handle {
        Ok(handle) => handle,
        Err(e) => return Err(io::Error::from_raw_os_error(e.code().0)),
    };
    
    // Send frames
    for frame in &frames {
        write_u32(pipe_handle, frame.len() as u32)?;
        write_exact(pipe_handle, frame)?;
    }
    
    // Read response: count + lengths
    let count = read_u32(pipe_handle)?;
    let mut lengths = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let len = read_u32(pipe_handle)?;
        lengths.push(len);
    }
    
    // SAFETY: We're calling CloseHandle with a valid handle.
    unsafe { let _ = CloseHandle(pipe_handle); };
    
    Ok(lengths)
}

pub fn named_pipe_frame_lengths(pipe_name: &str, frames: &[Vec<u8>]) -> io::Result<Vec<u32>> {
    let (tx, rx) = mpsc::channel();
    let n_frames = frames.len();
    
    // Convert borrowed data to owned for thread safety
    let pipe_name_owned = pipe_name.to_string();
    let frames_owned = frames.to_vec();
    
    let server_handle = thread::spawn(move || {
        let result = server(pipe_name_owned, n_frames);
        tx.send(()).unwrap_or_default(); // Signal that server is done
        result
    });
    
    // Give server time to create pipe
    thread::sleep(std::time::Duration::from_millis(100));
    
    let client_handle = thread::spawn(move || {
        client(pipe_name.to_string(), frames_owned)
    });
    
    // Wait for server to finish
    rx.recv().unwrap_or_default();
    
    let server_result = server_handle.join().unwrap_or_else(|e| {
        Err(io::Error::new(io::ErrorKind::Other, format!("Server thread panicked: {:?}", e)))
    });
    
    let client_result = client_handle.join().unwrap_or_else(|e| {
        Err(io::Error::new(io::ErrorKind::Other, format!("Client thread panicked: {:?}", e)))
    });
    
    server_result?;
    client_result
}