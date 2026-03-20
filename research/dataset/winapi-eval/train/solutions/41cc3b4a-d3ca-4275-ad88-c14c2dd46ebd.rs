use std::ffi::OsStr;
use std::io;
use std::iter::once;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::sync::mpsc;
use std::thread;

use windows::core::{Error, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ, FILE_MAP_WRITE,
    PAGE_READWRITE,
};
use windows::Win32::System::Threading::{CreateEventW, SetEvent, WaitForSingleObject};

const BUFFER_SIZE: usize = 4096;
const TIMEOUT_MS: u32 = 5000;

#[repr(C)]
struct SharedMemory {
    write_len: u32,
    read_len: u32,
    buffer: [u8; BUFFER_SIZE],
}

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn to_io_error(e: Error) -> io::Error {
    io::Error::other(e)
}

pub fn shm_event_handshake(base_name: &str, msg: &[u8]) -> io::Result<Vec<u8>> {
    if msg.len() > BUFFER_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Message too large for buffer",
        ));
    }

    let file_mapping_name = format!("{}_shm", base_name);
    let data_ready_name = format!("{}_data_ready", base_name);
    let ack_ready_name = format!("{}_ack_ready", base_name);

    let file_mapping_w = wide_null(OsStr::new(&file_mapping_name));
    let data_ready_w = wide_null(OsStr::new(&data_ready_name));
    let ack_ready_w = wide_null(OsStr::new(&ack_ready_name));

    // Create shared memory
    let file_mapping = unsafe {
        CreateFileMappingW(
            HANDLE::default(),
            None,
            PAGE_READWRITE,
            0,
            mem::size_of::<SharedMemory>() as u32,
            PCWSTR(file_mapping_w.as_ptr()),
        )
    }
    .map_err(to_io_error)?;

    let view = unsafe {
        MapViewOfFile(
            file_mapping,
            FILE_MAP_READ | FILE_MAP_WRITE,
            0,
            0,
            mem::size_of::<SharedMemory>(),
        )
    };

    if view.Value.is_null() {
        let _ = unsafe { CloseHandle(file_mapping) };
        return Err(io::Error::other("Failed to map view of file"));
    }

    let shared_mem = view.Value as *mut SharedMemory;

    // Initialize shared memory
    unsafe {
        (*shared_mem).write_len = 0;
        (*shared_mem).read_len = 0;
        ptr::write_bytes((*shared_mem).buffer.as_mut_ptr(), 0, BUFFER_SIZE);
    }

    // Create events
    let data_ready = unsafe { CreateEventW(None, false, false, PCWSTR(data_ready_w.as_ptr())) }
        .map_err(|e| {
            let _ = unsafe { UnmapViewOfFile(view) };
            let _ = unsafe { CloseHandle(file_mapping) };
            to_io_error(e)
        })?;

    let ack_ready = unsafe { CreateEventW(None, false, false, PCWSTR(ack_ready_w.as_ptr())) }
        .map_err(|e| {
            let _ = unsafe { CloseHandle(data_ready) };
            let _ = unsafe { UnmapViewOfFile(view) };
            let _ = unsafe { CloseHandle(file_mapping) };
            to_io_error(e)
        })?;

    // Channel for consumer to send received bytes
    let (tx, rx) = mpsc::channel();

    // Convert handles to raw values that can be sent between threads
    let data_ready_raw = data_ready.0 as usize;
    let ack_ready_raw = ack_ready.0 as usize;
    let shared_mem_raw = shared_mem as usize;

    // Consumer thread
    let consumer_handle = thread::spawn(move || -> io::Result<()> {
        // Reconstruct handles from raw values
        let data_ready_handle = HANDLE(data_ready_raw as *mut std::ffi::c_void);
        let ack_ready_handle = HANDLE(ack_ready_raw as *mut std::ffi::c_void);
        let shared_mem_ptr = shared_mem_raw as *mut SharedMemory;

        // Wait for data_ready event
        let wait_result = unsafe { WaitForSingleObject(data_ready_handle, TIMEOUT_MS) };

        if wait_result != WAIT_OBJECT_0 {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Timeout waiting for data",
            ));
        }

        // Read message from shared memory
        let write_len = unsafe { (*shared_mem_ptr).write_len } as usize;
        if write_len > BUFFER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid write_len in shared memory",
            ));
        }

        let mut received = vec![0u8; write_len];
        unsafe {
            ptr::copy_nonoverlapping(
                (*shared_mem_ptr).buffer.as_ptr(),
                received.as_mut_ptr(),
                write_len,
            );
        }

        // Set read_len and signal ack_ready
        unsafe {
            (*shared_mem_ptr).read_len = write_len as u32;
        }

        unsafe { SetEvent(ack_ready_handle) }.map_err(io::Error::other)?;

        // Send received bytes back to main thread
        tx.send(received).map_err(io::Error::other)?;

        Ok(())
    });

    // Producer (main thread)
    // Write message to shared memory
    unsafe {
        ptr::copy_nonoverlapping(msg.as_ptr(), (*shared_mem).buffer.as_mut_ptr(), msg.len());
        (*shared_mem).write_len = msg.len() as u32;
    }

    // Signal data_ready
    unsafe { SetEvent(data_ready) }.map_err(|e| {
        let _ = unsafe { CloseHandle(ack_ready) };
        let _ = unsafe { CloseHandle(data_ready) };
        let _ = unsafe { UnmapViewOfFile(view) };
        let _ = unsafe { CloseHandle(file_mapping) };
        to_io_error(e)
    })?;

    // Wait for ack_ready
    let wait_result = unsafe { WaitForSingleObject(ack_ready, TIMEOUT_MS) };
    if wait_result != WAIT_OBJECT_0 {
        let _ = unsafe { CloseHandle(ack_ready) };
        let _ = unsafe { CloseHandle(data_ready) };
        let _ = unsafe { UnmapViewOfFile(view) };
        let _ = unsafe { CloseHandle(file_mapping) };
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "Timeout waiting for acknowledgment",
        ));
    }

    // Get received bytes from consumer
    let received = rx.recv().map_err(io::Error::other)?;

    // Wait for consumer thread to finish
    consumer_handle
        .join()
        .map_err(|e| io::Error::other(format!("{:?}", e)))??;

    // Cleanup
    let _ = unsafe { CloseHandle(ack_ready) };
    let _ = unsafe { CloseHandle(data_ready) };
    let _ = unsafe { UnmapViewOfFile(view) };
    let _ = unsafe { CloseHandle(file_mapping) };

    Ok(received)
}
