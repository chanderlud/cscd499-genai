use std::path::Path;
use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND};
use windows::Win32::Storage::FileSystem::{
    ReplaceFileW, MoveFileExW, MOVEFILE_REPLACE_EXISTING, REPLACE_FILE_FLAGS,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(once(0)).collect()
}

pub fn replace_file(src: &Path, dst: &Path) -> Result<()> {
    let src_w = wide_null(src.as_os_str());
    let dst_w = wide_null(dst.as_os_str());

    unsafe {
        ReplaceFileW(
            PCWSTR(dst_w.as_ptr()),
            PCWSTR(src_w.as_ptr()),
            None,
            REPLACE_FILE_FLAGS(0),
            None,
            None,
        )
    }
    .or_else(|err| {
        let win32_error = err.win32_error();
        if win32_error == Some(ERROR_FILE_NOT_FOUND) || win32_error == Some(ERROR_PATH_NOT_FOUND) {
            unsafe {
                MoveFileExW(
                    PCWSTR(src_w.as_ptr()),
                    PCWSTR(dst_w.as_ptr()),
                    MOVEFILE_REPLACE_EXISTING,
                )
            }
        } else {
            Err(err)
        }
    })
}
