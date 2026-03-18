use std::path::Path;
use windows::core::{Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::{
    CopyFileExW, COPYFILE_FLAGS, COPYPROGRESSROUTINE_PROGRESS, LPPROGRESS_ROUTINE_CALLBACK_REASON,
    PROGRESS_CANCEL, PROGRESS_CONTINUE,
};

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
    _callback_reason: LPPROGRESS_ROUTINE_CALLBACK_REASON,
    _source_file: HANDLE,
    _destination_file: HANDLE,
    data: *const core::ffi::c_void,
) -> COPYPROGRESSROUTINE_PROGRESS {
    // SAFETY: Caller ensures data pointer is valid for the duration of the copy operation
    if data.is_null() {
        return PROGRESS_CONTINUE;
    }

    let progress_data = &mut *(data as *mut CopyProgressData);
    progress_data.transferred = total_bytes_transferred as u64;

    if let Some(threshold) = progress_data.cancel_after {
        if progress_data.transferred >= threshold {
            return PROGRESS_CANCEL;
        }
    }

    PROGRESS_CONTINUE
}

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn copy_with_progress(src: &Path, dst: &Path, cancel_after: Option<u64>) -> Result<u64> {
    // Check if source and destination are the same file
    if src == dst {
        return Err(windows::core::Error::from_hresult(
            HRESULT::from_win32(80), // ERROR_FILE_EXISTS
        ));
    }

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
            Some(&mut progress_data as *mut _ as *const core::ffi::c_void),
            None,
            COPYFILE_FLAGS(0),
        )
    };

    match result {
        Ok(()) => Ok(progress_data.transferred),
        Err(error) => Err(error),
    }
}
