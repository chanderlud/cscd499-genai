use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use windows::core::{Result, GUID};
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{SHGetKnownFolderPath, KNOWN_FOLDER_FLAG};

pub fn known_folder(id: GUID) -> Result<PathBuf> {
    // SAFETY: FFI call to Windows Shell API
    let path_ptr = unsafe { SHGetKnownFolderPath(&id, KNOWN_FOLDER_FLAG(0), None) }?;

    // SAFETY: path_ptr is valid on success, guaranteed null-terminated
    let wide_path = unsafe {
        let mut len = 0;
        while *path_ptr.0.add(len) != 0 {
            len += 1;
        }
        std::slice::from_raw_parts(path_ptr.0, len)
    };

    let os_string = OsString::from_wide(wide_path);
    let path = PathBuf::from(os_string);

    // SAFETY: Free the allocated memory using CoTaskMemFree
    unsafe { CoTaskMemFree(Some(path_ptr.0 as *const _)) };

    Ok(path)
}
