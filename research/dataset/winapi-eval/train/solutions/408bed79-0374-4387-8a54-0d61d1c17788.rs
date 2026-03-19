use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use windows::core::{Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{ERROR_NO_MORE_FILES, HANDLE};
use windows::Win32::Storage::FileSystem::{
    FindClose, FindFirstFileW, FindNextFileW, WIN32_FIND_DATAW,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

struct FindHandle(HANDLE);

impl Drop for FindHandle {
    fn drop(&mut self) {
        // SAFETY: The handle is valid when created and we own it exclusively.
        unsafe {
            let _ = FindClose(self.0);
        }
    }
}

pub fn list_dir(path: &Path) -> Result<Vec<OsString>> {
    // Build search pattern: path\*
    let mut pattern = path.as_os_str().to_owned();
    pattern.push(std::path::MAIN_SEPARATOR_STR);
    pattern.push("*");

    let pattern_wide = wide_null(&pattern);
    let mut find_data = WIN32_FIND_DATAW::default();

    // SAFETY: pattern_wide is a valid null-terminated wide string, find_data is valid for output.
    let handle = unsafe { FindFirstFileW(PCWSTR(pattern_wide.as_ptr()), &mut find_data) }?;
    let _guard = FindHandle(handle);

    let mut entries = Vec::new();

    loop {
        // Convert the found file name to OsString
        let name_len = find_data
            .cFileName
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(find_data.cFileName.len());
        let name = OsString::from_wide(&find_data.cFileName[..name_len]);

        // Exclude "." and ".."
        if name != "." && name != ".." {
            entries.push(name);
        }

        // SAFETY: handle is valid, find_data is valid for output.
        let result = unsafe { FindNextFileW(handle, &mut find_data) };
        if let Err(e) = result {
            if e.code() == HRESULT::from_win32(ERROR_NO_MORE_FILES.0) {
                break;
            } else {
                return Err(e);
            }
        }
    }

    entries.sort();
    Ok(entries)
}
