use std::path::Path;
use windows::core::{Result, PCWSTR, PWSTR, BOOL, HRESULT};
use windows::Win32::Foundation::{LocalFree, HLOCAL, ERROR_INVALID_PARAMETER, ERROR_ACCESS_DENIED};
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW,
    ConvertSecurityDescriptorToStringSecurityDescriptorW,
    SDDL_REVISION_1,
};
use windows::Win32::Security::{
    SetFileSecurityW, GetFileSecurityW, DACL_SECURITY_INFORMATION,
    PSECURITY_DESCRIPTOR, OWNER_SECURITY_INFORMATION, GROUP_SECURITY_INFORMATION,
};
use windows::Win32::Storage::FileSystem::{GetFileAttributesW, FILE_ATTRIBUTE_READONLY, INVALID_FILE_ATTRIBUTES};

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
    // Check for empty SDDL string
    if sddl.is_empty() {
        return Err(windows::core::Error::from_hresult(
            HRESULT::from_win32(ERROR_INVALID_PARAMETER.0)
        ));
    }

    let path_w = wide_null(path.as_os_str());
    let sddl_w = wide_null(std::ffi::OsStr::new(sddl));

    // Check if file is read-only
    let attrs = unsafe { GetFileAttributesW(PCWSTR(path_w.as_ptr())) };
    if attrs == INVALID_FILE_ATTRIBUTES {
        return Err(windows::core::Error::from_thread());
    }
    // Fix: Use .0 to access inner u32 value for bitwise operation
    if attrs & FILE_ATTRIBUTE_READONLY.0 != 0 {
        return Err(windows::core::Error::from_hresult(
            HRESULT::from_win32(ERROR_ACCESS_DENIED.0)
        ));
    }

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

    // Query the security descriptor back from the file
    let mut needed_size: u32 = 0;
    
    // First call to get required buffer size
    // SAFETY: FFI call with null buffer to get size
    unsafe {
        GetFileSecurityW(
            PCWSTR(path_w.as_ptr()),
            DACL_SECURITY_INFORMATION.0,
            None,
            0,
            &mut needed_size,
        );
    }
    
    // Allocate buffer for security descriptor
    let mut buffer = vec![0u8; needed_size as usize];
    let sd_query_ptr = PSECURITY_DESCRIPTOR(buffer.as_mut_ptr() as *mut _);
    
    // Second call to actually get the security descriptor
    // SAFETY: FFI call with properly allocated buffer
    let result = unsafe {
        GetFileSecurityW(
            PCWSTR(path_w.as_ptr()),
            DACL_SECURITY_INFORMATION.0,
            Some(sd_query_ptr),
            needed_size,
            &mut needed_size,
        )
    };
    check_bool(result)?;
    
    // Convert security descriptor back to SDDL string
    let mut sddl_ptr: PWSTR = PWSTR(std::ptr::null_mut());
    let mut sddl_len: u32 = 0;
    
    // SAFETY: FFI call with valid security descriptor
    unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            sd_query_ptr,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut sddl_ptr,
            Some(&mut sddl_len),
        )?;
    }
    
    // Convert the returned SDDL string to Rust String
    let result_sddl = if !sddl_ptr.0.is_null() {
        // SAFETY: We know sddl_ptr is valid and null-terminated
        unsafe {
            // sddl_len includes the null terminator, so subtract 1
            let wide_slice = std::slice::from_raw_parts(sddl_ptr.0, (sddl_len - 1) as usize);
            String::from_utf16_lossy(wide_slice)
        }
    } else {
        String::new()
    };
    
    // Clean up the SDDL string allocated by ConvertSecurityDescriptorToStringSecurityDescriptorW
    // SAFETY: Freeing memory allocated by ConvertSecurityDescriptorToStringSecurityDescriptorW
    unsafe {
        if !sddl_ptr.0.is_null() {
            let _ = LocalFree(Some(HLOCAL(sddl_ptr.0 as *mut _)));
        }
    }
    
    Ok(result_sddl)
}