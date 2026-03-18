use std::path::Path;
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND};
use windows::Win32::Storage::FileSystem::{
    MOVEFILE_REPLACE_EXISTING, MoveFileExW, REPLACE_FILE_FLAGS, ReplaceFileW,
};
use windows::core::{PCWSTR, Result};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(once(0)).collect()
}

fn win32_code(err: &windows::core::Error) -> u32 {
    // windows::core::Error stores HRESULTs like 0x80070002.
    // The underlying Win32 code is in the low 16 bits.
    (err.code().0 as u32) & 0xFFFF
}

pub fn replace_file(src: &Path, dst: &Path) -> Result<()> {
    // Make the wrapper treat identical paths as a no-op.
    if src == dst {
        return Ok(());
    }

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
            let code = win32_code(&err);

            if code == ERROR_FILE_NOT_FOUND.0 || code == ERROR_PATH_NOT_FOUND.0 {
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
