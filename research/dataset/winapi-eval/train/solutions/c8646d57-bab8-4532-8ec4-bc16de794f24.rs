use std::path::Path;
use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Storage::FileSystem::{
    CreateFileW, GetFileSizeEx, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_SHARE_READ,
    OPEN_EXISTING,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(once(0)).collect()
}

pub fn file_size(path: &Path) -> Result<u64> {
    let wide_path = wide_null(path.as_os_str());

    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_GENERIC_READ.0,
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }?;

    let mut size = 0i64;
    let result = unsafe { GetFileSizeEx(handle, &mut size) };

    // Always close the handle and propagate any error
    unsafe { CloseHandle(handle) }?;

    // Check result properly
    match result {
        Ok(()) => Ok(size as u64),
        Err(e) => Err(e),
    }
}
