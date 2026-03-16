use std::ffi::OsStr;
use std::io;
use std::iter::once;
use std::mem;
use std::os::windows::ffi::OsStrExt;
use std::ptr;
use std::slice;
use std::sync::mpsc;
use std::thread;

use windows::core::{Error, Result, HRESULT, PCWSTR};
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
    io::Error::new(io::ErrorKind::Other, e)
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

    if file_mapping.is_invalid() {
        return Err(Error::from_win32().into());
    }

    let view = unsafe {
        MapViewOfFile(
            file_mapping,
            FILE_MAP_READ | FILE_MAP_WRITE,
            0,
            0,
            mem::size_of::<SharedMemory>(),
        )
    };

    if view.is_null() {
        unsafe { CloseHandle(file_mapping) };
        return Err(Error::from_win32().into());
    }

    let shared_mem = view as *mut SharedMemory;

    // Initialize shared memory
    unsafe {
        (*shared_mem).write_len = 0;
        (*shared_mem).read_len = 0;
        ptr::write_bytes((*shared_mem).buffer.as_mut_ptr(), 0, BUFFER_SIZE);
    }

    // Create events
    let data_ready = unsafe {
        CreateEventW(None, false, false, PCWSTR(data_ready_w.as_ptr()))
    }
    .map_err(|e| {
        unsafe {
            UnmapViewOfFile(view);
            CloseHandle(file_mapping);
        }
        to_io_error(e)
    })?;

    let ack_ready = unsafe {
        CreateEventW(None, false, false, PCWSTR(ack_ready_w.as_ptr()))
    }
    .map_err(|e| {
        unsafe {
            CloseHandle(data_ready);
            UnmapViewOfFile(view);
            CloseHandle(file_mapping);
        }
        to_io_error(e)
    })?;

    // Channel for consumer to send received bytes
    let (tx, rx) = mpsc::channel();

    // Consumer thread
    let consumer_data_ready = data_ready;
    let consumer_ack_ready = ack_ready;
    let consumer_shared_mem = shared_mem;
    let consumer_handle = thread::spawn(move || -> io::Result<()> {
        // Wait for data_ready event
        let wait_result = unsafe {
            WaitForSingleObject(consumer_data_ready, TIMEOUT_MS)
        };

        if wait_result != WAIT_OBJECT_0 {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Timeout waiting for data",
            ));
        }

        // Read message from shared memory
        let write_len = unsafe { (*consumer_shared_mem).write_len } as usize;
        if write_len > BUFFER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid write_len in shared memory",
            ));
        }

        let mut received = vec![0u8; write_len];
        unsafe {
            ptr::copy_nonoverlapping(
                (*consumer_shared_mem).buffer.as_ptr(),
                received.as_mut_ptr(),
                write_len,
            );
        }

        // Set read_len and signal ack_ready
        unsafe {
            (*consumer_shared_mem).read_len = write_len as u32;
        }

        unsafe { SetEvent(consumer_ack_ready) }
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Send received bytes back to main thread
        tx.send(received).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, e)
        })?;

        Ok(())
    });

    // Producer (main thread)
    // Write message to shared memory
    unsafe {
        ptr::copy_nonoverlapping(
            msg.as_ptr(),
            (*shared_mem).buffer.as_mut_ptr(),
            msg.len(),
        );
        (*shared_mem).write_len = msg.len() as u32;
    }

    // Signal data_ready
    unsafe { SetEvent(data_ready) }.map_err(|e| {
        unsafe {
            CloseHandle(ack_ready);
            CloseHandle(data_ready);
            UnmapViewOfFile(view);
            CloseHandle(file_mapping);
        }
        to_io_error(e)
    })?;

    // Wait for ack_ready
    let wait_result = unsafe { WaitForSingleObject(ack_ready, TIMEOUT_MS) };
    if wait_result != WAIT_OBJECT_0 {
        unsafe {
            CloseHandle(ack_ready);
            CloseHandle(data_ready);
            UnmapViewOfFile(view);
            CloseHandle(file_mapping);
        }
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "Timeout waiting for acknowledgment",
        ));
    }

    // Get received bytes from consumer
    let received = rx.recv().map_err(|e| {
        io::Error::new(io::ErrorKind::Other, e)
    })??;

    // Wait for consumer thread to finish
    consumer_handle.join().map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("{:?}", e))
    })??;

    // Cleanup
    unsafe {
        CloseHandle(ack_ready);
        CloseHandle(data_ready);
        UnmapViewOfFile(view);
        CloseHandle(file_mapping);
    }

    Ok(received)
}