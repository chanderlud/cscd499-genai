use windows::core::{Result, Error, PCWSTR, HRESULT};
use windows::Win32::Foundation::{HANDLE, CloseHandle, ERROR_INVALID_PARAMETER};
use windows::Win32::Security::{
    AllocateAndInitializeSid, FreeSid, GetSidLengthRequired, GetSidSubAuthorityCount,
    PSID, SECURITY_NT_AUTHORITY
};
use windows::Win32::System::Threading::{
    CreateBoundaryDescriptorW, AddSIDToBoundaryDescriptor, DeleteBoundaryDescriptor,
    CreatePrivateNamespaceW, ClosePrivateNamespace, PRIVATE_NAMESPACE_FLAG_DESTROY,
    CreateMutexW, OpenMutexW, MUTEX_ALL_ACCESS
};
use windows::Win32::System::SystemServices::{
    SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_USERS
};
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn get_current_user_sid() -> Result<Vec<u8>> {
    // Create a SID for the "Users" group (S-1-5-32-545)
    let sid_authority = SECURITY_NT_AUTHORITY;
    let mut sid = PSID::default();
    
    // AllocateAndInitializeSid returns Result<()>, so we can use ?
    unsafe {
        AllocateAndInitializeSid(
            &sid_authority,
            2,
            SECURITY_BUILTIN_DOMAIN_RID as u32,
            DOMAIN_ALIAS_RID_USERS as u32,
            0, 0, 0, 0, 0, 0,
            &mut sid,
        )?;
    }
    
    // Get the size of the SID
    let sid_size = unsafe { 
        GetSidLengthRequired(*GetSidSubAuthorityCount(sid))
    };
    let mut sid_buffer = vec![0u8; sid_size as usize];
    
    // Copy the SID to our buffer
    unsafe {
        std::ptr::copy_nonoverlapping(
            sid.0 as *const u8,
            sid_buffer.as_mut_ptr(),
            sid_size as usize,
        );
        FreeSid(sid);
    }
    
    Ok(sid_buffer)
}

pub fn private_namespace_mutex_roundtrip(ns_name: &str, mutex_name: &str) -> Result<bool> {
    // Check for embedded NUL characters in input strings
    if ns_name.contains('\0') {
        return Err(Error::from_hresult(HRESULT::from_win32(ERROR_INVALID_PARAMETER.0)));
    }
    if mutex_name.contains('\0') {
        return Err(Error::from_hresult(HRESULT::from_win32(ERROR_INVALID_PARAMETER.0)));
    }
    
    // 1. Get current user's SID
    let user_sid = get_current_user_sid()?;
    
    // 2. Create boundary descriptor
    let boundary_name = wide_null(OsStr::new(ns_name));
    let boundary_descriptor = unsafe {
        CreateBoundaryDescriptorW(PCWSTR(boundary_name.as_ptr()), 0)
    };
    
    // Check for error (HANDLE(0) indicates failure)
    if boundary_descriptor == HANDLE(ptr::null_mut()) {
        return Err(Error::from_thread());
    }
    
    // 3. Add user SID to boundary descriptor
    unsafe {
        AddSIDToBoundaryDescriptor(
            &boundary_descriptor as *const HANDLE as *mut HANDLE,
            PSID(user_sid.as_ptr() as *mut _)
        )?;
    }
    
    // 4. Create private namespace
    let ns_name_wide = wide_null(OsStr::new(ns_name));
    let namespace = unsafe {
        CreatePrivateNamespaceW(
            None,
            boundary_descriptor.0 as *const core::ffi::c_void,
            PCWSTR(ns_name_wide.as_ptr())
        )
    };
    
    // Check for error (HANDLE(0) indicates failure)
    if namespace == HANDLE(ptr::null_mut()) {
        unsafe { DeleteBoundaryDescriptor(boundary_descriptor); }
        return Err(Error::from_thread());
    }
    
    // 5. Create mutex in the namespace - ADD NAMESPACE PREFIX
    let full_mutex_name = format!("{}\\{}", ns_name, mutex_name);
    let mutex_name_wide = wide_null(OsStr::new(&full_mutex_name));
    let mutex_handle = unsafe {
        CreateMutexW(
            None,
            false,
            PCWSTR(mutex_name_wide.as_ptr())
        )
    }?;
    
    // 6. Try to open the same mutex - USE SAME PREFIXED NAME
    let open_result = unsafe {
        OpenMutexW(
            MUTEX_ALL_ACCESS,
            false,
            PCWSTR(mutex_name_wide.as_ptr())
        )
    };
    
    let success = open_result.is_ok();
    
    // 7. Cleanup
    if let Ok(handle) = open_result {
        unsafe { CloseHandle(handle)?; }
    }
    unsafe { CloseHandle(mutex_handle)?; }
    unsafe { ClosePrivateNamespace(namespace, PRIVATE_NAMESPACE_FLAG_DESTROY); }
    unsafe { DeleteBoundaryDescriptor(boundary_descriptor); }
    
    Ok(success)
}