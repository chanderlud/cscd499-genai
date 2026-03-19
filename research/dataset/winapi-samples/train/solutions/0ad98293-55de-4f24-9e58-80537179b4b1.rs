use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, GetFinalPathNameByHandleW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_NAME_NORMALIZED, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
    GETFINALPATHNAMEBYHANDLE_FLAGS, OPEN_EXISTING, VOLUME_NAME_DOS,
};

pub fn final_path(path: &Path) -> Result<PathBuf> {
    // Convert input path to wide string on stack
    let mut wide_path = [0u16; 1024];
    let mut path_len = 0;

    for (i, unit) in path.as_os_str().encode_wide().enumerate() {
        if i >= 1023 {
            return Err(Error::from_hresult(windows::core::HRESULT::from_win32(206)));
            // ERROR_FILENAME_EXCED_RANGE
        }
        wide_path[i] = unit;
        path_len = i + 1;
    }
    wide_path[path_len] = 0; // Null terminator

    // Open the file with backup semantics to allow opening directories
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            0, // No access needed, just for querying
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )
    }?;

    // Ensure handle is closed even if we return early
    struct HandleGuard(HANDLE);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            let _ = unsafe { CloseHandle(self.0) };
        }
    }
    let _guard = HandleGuard(handle);

    // Get final path on stack buffer
    let mut final_path_wide = [0u16; 1024];
    let result = unsafe {
        GetFinalPathNameByHandleW(
            handle,
            &mut final_path_wide,
            GETFINALPATHNAMEBYHANDLE_FLAGS(FILE_NAME_NORMALIZED.0 | VOLUME_NAME_DOS.0),
        )
    };

    if result == 0 {
        return Err(Error::from_thread());
    }

    if result >= 1024 {
        return Err(Error::from_hresult(windows::core::HRESULT::from_win32(206)));
        // ERROR_FILENAME_EXCED_RANGE
    }

    // Convert wide string to PathBuf
    let len = result as usize;
    let path_slice = &final_path_wide[..len];
    let os_string = std::ffi::OsString::from_wide(path_slice);
    Ok(PathBuf::from(os_string))
}
