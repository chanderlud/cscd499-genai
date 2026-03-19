use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, ERROR_FILE_NOT_FOUND};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FlushFileBuffers, GetTempFileNameW, MoveFileExW, ReplaceFileW, WriteFile,
    CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_WRITE_THROUGH, FILE_GENERIC_WRITE,
    FILE_SHARE_NONE, MOVEFILE_REPLACE_EXISTING, REPLACE_FILE_FLAGS,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn get_directory(path: &Path) -> Result<Vec<u16>> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    Ok(wide_null(parent.as_os_str()))
}

pub fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    let dir_wide = get_directory(path)?;

    let mut temp_path = [0u16; 260];
    let prefix = wide_null(OsStr::new("tmp"));

    let result = unsafe {
        GetTempFileNameW(
            PCWSTR(dir_wide.as_ptr()),
            PCWSTR(prefix.as_ptr()),
            0,
            &mut temp_path,
        )
    };

    if result == 0 {
        return Err(Error::from_thread());
    }

    let temp_handle = unsafe {
        CreateFileW(
            PCWSTR(temp_path.as_ptr()),
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_NONE,
            None,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_WRITE_THROUGH,
            None,
        )
    }?;

    let mut bytes_written = 0u32;
    let write_result = unsafe {
        WriteFile(
            temp_handle,
            Some(data),
            Some(&mut bytes_written as *mut u32),
            None,
        )
    };

    if write_result.is_err() {
        let err = Error::from_thread();
        unsafe {
            let _ = CloseHandle(temp_handle);
        }
        return Err(err);
    }

    let flush_result = unsafe { FlushFileBuffers(temp_handle) };
    if flush_result.is_err() {
        let err = Error::from_thread();
        unsafe {
            let _ = CloseHandle(temp_handle);
        }
        return Err(err);
    }

    unsafe { CloseHandle(temp_handle) }?;

    let dest_wide = wide_null(path.as_os_str());

    let replace_result = unsafe {
        ReplaceFileW(
            PCWSTR(dest_wide.as_ptr()),
            PCWSTR(temp_path.as_ptr()),
            None,
            REPLACE_FILE_FLAGS(0),
            None,
            None,
        )
    };

    if replace_result.is_ok() {
        return Ok(());
    }

    let err = Error::from_thread();
    if err.code() == HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0) {
        let move_result = unsafe {
            MoveFileExW(
                PCWSTR(temp_path.as_ptr()),
                PCWSTR(dest_wide.as_ptr()),
                MOVEFILE_REPLACE_EXISTING,
            )
        };

        if move_result.is_ok() {
            return Ok(());
        }

        return Err(Error::from_thread());
    }

    Err(err)
}
