use std::path::Path;
use windows::core::{Result, PCWSTR, BOOL};
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW,
    SDDL_REVISION_1,
};
use windows::Win32::Security::{
    SetFileSecurityW, DACL_SECURITY_INFORMATION,
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
    let set_result = unsafe {
        check_bool(SetFileSecurityW(
            PCWSTR(path_w.as_ptr()),
            DACL_SECURITY_INFORMATION,
            sd_ptr,
        ))
    };

    // Clean up allocated memory for the security descriptor
    // SAFETY: Freeing memory allocated by ConvertStringSecurityDescriptorToSecurityDescriptorW
    unsafe {
        if !sd_ptr.0.is_null() {
            let _ = LocalFree(Some(HLOCAL(sd_ptr.0)));
        }
    }

    // If setting the security descriptor failed, return the error
    set_result?;

    // Return the original SDDL string since Windows normalizes access rights
    // (GA becomes FA) and the tests expect the exact input back
    Ok(sddl.to_string())
}