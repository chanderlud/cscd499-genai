use windows::core::{Result, Error, PCWSTR};
use windows::Win32::Foundation::{HANDLE, CloseHandle};
use windows::Win32::Storage::FileSystem::{CreateFileW, FILE_GENERIC_READ, FILE_SHARE_READ, OPEN_EXISTING, FILE_FLAG_OVERLAPPED, GetFileSizeEx, ReadFile};
use windows::Win32::System::IO::{CreateIoCompletionPort, GetQueuedCompletionStatus, OVERLAPPED};
use windows::Win32::System::Threading::INFINITE;
use std::ffi::OsStr;
use std::mem::zeroed;
use std::ptr::null_mut;

fn wide_null(s: &OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

#[repr(C)]
struct MyOverlapped {
    overlapped: OVERLAPPED,
    chunk_index: usize,
}

pub fn read_file_iocp(path: &std::path::Path, chunk_size: u32, max_in_flight: u32) -> Result<Vec<u8>> {
    // Open file with overlapped I/O
    let wide_path = wide_null(path.as_os_str());
    let file_handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_GENERIC_READ.0,
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_FLAG_OVERLAPPED,
            None,
        )
    }?;

    // Get file size
    let mut file_size: i64 = 0;
    unsafe { GetFileSizeEx(file_handle, &mut file_size)?; }
    let file_size = file_size as usize;

    if file_size == 0 {
        unsafe { CloseHandle(file_handle)?; }
        return Ok(Vec::new());
    }

    // Create IO completion port
    let iocp_handle = unsafe {
        CreateIoCompletionPort(file_handle, None, 0, 0)
    }?;

    // Calculate number of chunks
    let chunk_size = chunk_size as usize;
    let max_in_flight = max_in_flight as usize;
    let num_chunks = (file_size + chunk_size - 1) / chunk_size;
    let mut result = vec![0u8; file_size];
    let mut completed_chunks = 0;
    let mut next_chunk_to_read = 0;
    
    // Track which chunks have been read
    let mut chunk_status = vec![false; num_chunks];
    
    // Issue initial reads
    while next_chunk_to_read < num_chunks && next_chunk_to_read < max_in_flight {
        issue_read(file_handle, next_chunk_to_read, chunk_size, file_size, &mut result)?;
        next_chunk_to_read += 1;
    }

    // Process completions
    while completed_chunks < num_chunks {
        let mut bytes_transferred: u32 = 0;
        let mut completion_key: usize = 0;
        let mut overlapped_ptr: *mut OVERLAPPED = null_mut();
        
        let success = unsafe {
            GetQueuedCompletionStatus(
                iocp_handle,
                &mut bytes_transferred,
                &mut completion_key,
                &mut overlapped_ptr,
                INFINITE,
            )
        };

        if success.is_ok() {
            // Get chunk index from custom overlapped structure
            let my_overlapped_ptr = overlapped_ptr as *mut MyOverlapped;
            let chunk_index = unsafe { (*my_overlapped_ptr).chunk_index };
            
            // Mark chunk as completed
            if !chunk_status[chunk_index] {
                chunk_status[chunk_index] = true;
                completed_chunks += 1;
                
                // Issue next read if available
                if next_chunk_to_read < num_chunks {
                    issue_read(file_handle, next_chunk_to_read, chunk_size, file_size, &mut result)?;
                    next_chunk_to_read += 1;
                }
            }
            
            // Clean up custom overlapped structure
            unsafe { drop(Box::from_raw(my_overlapped_ptr)); }
        } else {
            let err = Error::from_thread();
            unsafe {
                CloseHandle(file_handle)?;
                CloseHandle(iocp_handle)?;
            }
            return Err(err);
        }
    }

    // Clean up handles
    unsafe {
        CloseHandle(file_handle)?;
        CloseHandle(iocp_handle)?;
    }

    Ok(result)
}

fn issue_read(
    file_handle: HANDLE,
    chunk_index: usize,
    chunk_size: usize,
    file_size: usize,
    buffer: &mut [u8],
) -> Result<()> {
    let offset = chunk_index * chunk_size;
    let bytes_to_read = std::cmp::min(chunk_size, file_size - offset);
    
    // Create custom overlapped structure with chunk index
    let mut my_overlapped = Box::new(MyOverlapped {
        overlapped: unsafe { zeroed() },
        chunk_index,
    });
    my_overlapped.overlapped.Anonymous.Anonymous.Offset = offset as u32;
    my_overlapped.overlapped.Anonymous.Anonymous.OffsetHigh = (offset >> 32) as u32;
    
    // Box the custom overlapped structure to keep it alive
    let my_overlapped_ptr = Box::into_raw(my_overlapped);
    let overlapped_ptr = unsafe { &mut (*my_overlapped_ptr).overlapped as *mut OVERLAPPED };
    
    // Issue the read
    let result = unsafe {
        ReadFile(
            file_handle,
            Some(&mut buffer[offset..offset + bytes_to_read]),
            None,
            Some(overlapped_ptr),
        )
    };

    match result {
        Ok(()) => Ok(()),  // Operation completed immediately
        Err(e) if e.code() == windows::Win32::Foundation::ERROR_IO_PENDING.to_hresult() => Ok(()),  // Pending is OK
        Err(e) => {
            // Clean up the custom overlapped on error
            unsafe { drop(Box::from_raw(my_overlapped_ptr)); }
            Err(e)
        }
    }
}