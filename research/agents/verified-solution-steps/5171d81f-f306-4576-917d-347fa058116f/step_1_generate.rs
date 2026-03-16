use std::path::Path;
use windows::core::{Result, Error, PCWSTR};
use windows::Win32::Foundation::{BOOL, HANDLE};
use windows::Win32::Storage::FileSystem::{CopyFileExW, COPY_FILE_FLAGS};

trait ProgressFn {
    fn call(&self, total: u64, transferred: u64) -> bool;
}

impl<F: Fn(u64, u64) -> bool> ProgressFn for F {
    fn call(&self, total: u64, transferred: u64) -> bool {
        self(total, transferred)
    }
}

unsafe extern "system" fn progress_callback(
    total_file_size: i64,
    total_bytes_transferred: i64,
    _stream_size: i64,
    _stream_bytes_transferred: i64,
    _stream_number: u32,
    _callback_reason: u32,
    _source_file: HANDLE,
    _destination_file: HANDLE,
    data: *const core::ffi::c_void,
) -> i32 {
    // SAFETY: The caller ensures data points to a valid ProgressFn trait object
    // that lives for the duration of the copy operation.
    let progress = unsafe { &*(data as *const &dyn ProgressFn) };
    let should_continue = progress.call(total_file_size as u64, total_bytes_transferred as u64);
    if should_continue {
        0 // PROGRESS_CONTINUE
    } else {
        1 // PROGRESS_CANCEL
    }
}

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn copy_with_progress<F: Fn(u64, u64) -> bool>(
    src: &Path,
    dst: &Path,
    progress: F,
) -> Result<()> {
    let src_wide = wide_null(src.as_os_str());
    let dst_wide = wide_null(dst.as_os_str());
    
    let progress_ref: &dyn ProgressFn = &progress;
    let progress_data = &progress_ref as *const &dyn ProgressFn as *const core::ffi::c_void;
    
    // SAFETY: We're calling a Windows API function with valid parameters.
    // The progress callback is safe because:
    // 1. progress_data points to a valid trait object reference that lives for this call
    // 2. The callback only reads from the reference and calls the closure
    // 3. The closure is guaranteed to be valid for the duration of the copy
    let result = unsafe {
        CopyFileExW(
            PCWSTR(src_wide.as_ptr()),
            PCWSTR(dst_wide.as_ptr()),
            Some(progress_callback),
            Some(progress_data),
            None,
            COPY_FILE_FLAGS(0),
        )
    };
    
    if result.as_bool() {
        Ok(())
    } else {
        Err(Error::from_win32())
    }
}