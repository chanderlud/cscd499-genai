#![deny(warnings)]

use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::System::LibraryLoader::AddDllDirectory;

#[allow(dead_code)]
fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

#[allow(dead_code)]
fn call_add_dll_directory() -> Result<*mut core::ffi::c_void> {
    let path = wide_null(OsStr::new("C:\\Windows\\System32"));
    // SAFETY: AddDllDirectory expects a null-terminated wide string pointer.
    // `PCWSTR::from_raw(path.as_ptr())` safely wraps the pointer for FFI consumption.
    let cookie = unsafe { AddDllDirectory(PCWSTR::from_raw(path.as_ptr())) };
    if cookie.is_null() {
        return Err(Error::from_thread());
    }
    Ok(cookie)
}
