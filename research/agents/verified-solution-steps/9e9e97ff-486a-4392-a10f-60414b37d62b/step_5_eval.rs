use std::path::Path;
use windows::core::{Result, PCWSTR, PWSTR, BOOL, HRESULT};
use windows::Win32::Foundation::{LocalFree, HLOCAL, ERROR_INSUFFICIENT_BUFFER};
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW,
    ConvertSecurityDescriptorToStringSecurityDescriptorW,
    SDDL_REVISION_1,
};
use windows::Win32::Security::{
    GetFileSecurityW, SetFileSecurityW, DACL_SECURITY_INFORMATION,
    PSECURITY_DESCRIPTOR,
};

fn check_bool(result: BOOL) -> Result<()> {
    if result.as_bool() {
        Ok(())
    } else {
        Err(windows::core::Error::from_thread())
    }
}

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn set_get_file_sddl(path: &Path, sddl: &str) -> Result<String> {
    let path_w = wide_null(path.as_os_str());
    let sddl_w = wide_null(std::ffi::OsStr::new(sddl));

    // Convert SDDL string to security descriptor
    let mut sd_ptr = PSECURITY_DESCRIPTOR(std::ptr::null_mut());
    let mut sd_size: u32 = 0;
    // SAFETY: FFI call with valid pointers and sizes
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            PCWSTR(sddl_w.as_ptr()),
            SDDL_REVISION_1,
            &mut sd_ptr,
            Some(&mut sd_size),
        )?;
    }

    // Apply security descriptor to file
    // SAFETY: FFI call with valid path and security descriptor
    unsafe {
        check_bool(SetFileSecurityW(
            PCWSTR(path_w.as_ptr()),
            DACL_SECURITY_INFORMATION,
            sd_ptr,
        ))?;
    }

    // Query security descriptor from file
    let mut needed: u32 = 0;
    // First call to get required buffer size
    // SAFETY: FFI call with null buffer to get size
    unsafe {
        GetFileSecurityW(
            PCWSTR(path_w.as_ptr()),
            DACL_SECURITY_INFORMATION.0,
            None,
            0,
            &mut needed,
        );
    }
    let err = windows::core::Error::from_thread();
    if err.code() != HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0) {
        return Err(err);
    }

    let mut buffer = vec![0u8; needed as usize];
    // SAFETY: FFI call with properly sized buffer
    unsafe {
        check_bool(GetFileSecurityW(
            PCWSTR(path_w.as_ptr()),
            DACL_SECURITY_INFORMATION.0,
            Some(PSECURITY_DESCRIPTOR(buffer.as_mut_ptr() as *mut _)),
            needed,
            &mut needed,
        ))?;
    }

    // Convert security descriptor back to SDDL string
    let mut sddl_ptr: PWSTR = PWSTR(std::ptr::null_mut());
    let mut sddl_len: u32 = 0;
    // SAFETY: FFI call with valid security descriptor buffer
    unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            PSECURITY_DESCRIPTOR(buffer.as_mut_ptr() as *mut _),
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut sddl_ptr,
            Some(&mut sddl_len),
        )?;
    }

    // Convert wide string to Rust String
    // SAFETY: sddl_ptr is valid and null-terminated
    let result = unsafe {
        let len = (0..).take_while(|&i| *sddl_ptr.0.add(i) != 0).count();
        let slice = std::slice::from_raw_parts(sddl_ptr.0, len);
        String::from_utf16_lossy(slice)
    };

    // Clean up allocated memory
    // SAFETY: Freeing memory allocated by ConvertStringSecurityDescriptorToSecurityDescriptorW
    unsafe {
        if !sd_ptr.0.is_null() {
            let _ = LocalFree(Some(HLOCAL(sd_ptr.0)));
        }
        if !sddl_ptr.0.is_null() {
            let _ = LocalFree(Some(HLOCAL(sddl_ptr.0 as *mut _)));
        }
    }

    Ok(result)
}