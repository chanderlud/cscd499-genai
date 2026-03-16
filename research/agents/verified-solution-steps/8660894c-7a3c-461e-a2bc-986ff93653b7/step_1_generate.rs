use std::path::Path;
use windows::core::{Result, Error, HRESULT, BOOL, PCWSTR};
use windows::Win32::Foundation::{HANDLE, ERROR_REQUEST_ABORTED};
use windows::Win32::Storage::FileSystem::{CopyFileExW, COPY_FILE_FLAGS};

const PROGRESS_CONTINUE: u32 = 0;
const PROGRESS_CANCEL: u32 = 2;

struct CopyProgressData {
    cancel_after: Option<u64>,
    transferred: u64,
}

unsafe extern "system" fn progress_routine(
    _total_file_size: i64,
    total_bytes_transferred: i64,
    _stream_size: i64,
    _stream_bytes_transferred: i64,
    _stream_number: u32,
    _callback_reason: u32,
    _source_file: HANDLE,
    _destination_file: HANDLE,
    data: *const core::ffi::c_void,
) -> u32 {
    // SAFETY: Caller ensures data pointer is valid for the duration of the copy operation
    let progress_data = &mut *(data as *mut CopyProgressData);
    progress_data.transferred = total_bytes_transferred as u64;
    
    if let Some(threshold) = progress_data.cancel_after {
        if progress_data.transferred > threshold {
            return PROGRESS_CANCEL;
        }
    }
    
    PROGRESS_CONTINUE
}

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn copy_with_progress(src: &Path, dst: &Path, cancel_after: Option<u64>) -> Result<u64> {
    let src_wide = wide_null(src.as_os_str());
    let dst_wide = wide_null(dst.as_os_str());
    
    let mut progress_data = CopyProgressData {
        cancel_after,
        transferred: 0,
    };
    
    let result = unsafe {
        CopyFileExW(
            PCWSTR(src_wide.as_ptr()),
            PCWSTR(dst_wide.as_ptr()),
            Some(progress_routine),
            &mut progress_data as *mut _ as *const core::ffi::c_void,
            None,
            COPY_FILE_FLAGS(0),
        )
    };
    
    if result.as_bool() {
        Ok(progress_data.transferred)
    } else {
        let error = Error::from_thread();
        if error.code() == HRESULT::from_win32(ERROR_REQUEST_ABORTED.0) {
            Ok(progress_data.transferred)
        } else {
            Err(error)
        }
    }
}