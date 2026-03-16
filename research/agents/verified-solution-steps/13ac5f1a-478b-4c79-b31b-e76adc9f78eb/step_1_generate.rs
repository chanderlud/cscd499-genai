use std::path::PathBuf;
use windows::core::{Result, Error, GUID, PWSTR};
use windows::Win32::UI::Shell::{SHGetKnownFolderPath, CoTaskMemFree};
use windows::Win32::Foundation::S_OK;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

pub fn known_folder(id: GUID) -> Result<PathBuf> {
    let mut path_ptr = PWSTR::default();
    
    // SAFETY: FFI call to Windows Shell API
    let hr = unsafe { SHGetKnownFolderPath(&id, 0, None, &mut path_ptr) };
    if hr != S_OK {
        return Err(Error::from_hresult(hr));
    }
    
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