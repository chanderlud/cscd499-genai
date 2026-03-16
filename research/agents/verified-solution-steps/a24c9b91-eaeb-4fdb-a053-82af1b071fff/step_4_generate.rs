use windows::core::{Result, Error, PCWSTR, HRESULT};
use windows::Win32::Foundation::{HANDLE, HWND, HINSTANCE, CloseHandle, GetLastError};
use windows::Win32::Security::{
    CreateBoundaryDescriptorW, AddSIDToBoundaryDescriptor, 
    GetSidLengthRequired, GetSidSubAuthorityCount, GetSidSubAuthority,
    AllocateAndInitializeSid, FreeSid, SID_IDENTIFIER_AUTHORITY,
    SECURITY_NT_AUTHORITY
};
use windows::Win32::System::WindowsProgramming::{
    CreatePrivateNamespaceW, ClosePrivateNamespace, 
    PRIVATE_NAMESPACE_FLAG_DESTROY
};
use windows::Win32::System::Threading::{
    CreateMutexW, OpenMutexW, ReleaseMutex, MUTEX_ALL_ACCESS
};
use windows::Win32::System::Diagnostics::Debug::GetLastError;
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn get_current_user_sid() -> Result<Vec<u8>> {
    // Create a SID for the current user (simplified approach using well-known SID)
    let mut sid_authority = SID_IDENTIFIER_AUTHORITY {
        Value: SECURITY_NT_AUTHORITY,
    };
    let mut sid = null_mut();
    
    // Create a SID for the "Users" group (S-1-5-32-545)
    let result = unsafe {
        AllocateAndInitializeSid(
            &mut sid_authority,
            2,
            0x00000020, // SECURITY_BUILTIN_DOMAIN_RID
            0x00000221, // DOMAIN_ALIAS_RID_USERS
            0, 0, 0, 0, 0, 0,
            &mut sid,
        )
    };
    
    if result == false {
        return Err(Error::from_hresult(HRESULT::from_win32(unsafe { GetLastError().0 })));
    }
    
    // Get the size of the SID
    let sid_size = unsafe { GetSidLengthRequired(*GetSidSubAuthorityCount(sid)) };
    let mut sid_buffer = vec![0u8; sid_size as usize];
    
    // Copy the SID to our buffer
    unsafe {
        std::ptr::copy_nonoverlapping(
            sid as *const u8,
            sid_buffer.as_mut_ptr(),
            sid_size as usize,
        );
        FreeSid(sid);
    }
    
    Ok(sid_buffer)
}

pub fn private_namespace_mutex_roundtrip(ns_name: &str, mutex_name: &str) -> Result<bool> {
    // 1. Get current user's SID
    let user_sid = get_current_user_sid()?;
    
    // 2. Create boundary descriptor
    let boundary_name = wide_null(OsStr::new("MyBoundary"));
    let boundary_descriptor = unsafe {
        CreateBoundaryDescriptorW(PCWSTR(boundary_name.as_ptr()), 0)
    }?;
    
    // 3. Add user SID to boundary descriptor
    unsafe {
        AddSIDToBoundaryDescriptor(
            &boundary_descriptor,
            user_sid.as_ptr() as *mut _
        )?;
    }
    
    // 4. Create private namespace
    let ns_name_wide = wide_null(OsStr::new(ns_name));
    let namespace = unsafe {
        CreatePrivateNamespaceW(
            null_mut(),
            PCWSTR(ns_name_wide.as_ptr()),
            PRIVATE_NAMESPACE_FLAG_DESTROY,
            boundary_descriptor
        )
    }?;
    
    // 5. Create mutex in the namespace
    let mutex_name_wide = wide_null(OsStr::new(mutex_name));
    let mutex_handle = unsafe {
        CreateMutexW(
            null_mut(),
            false,
            PCWSTR(mutex_name_wide.as_ptr())
        )
    }?;
    
    // 6. Try to open the same mutex
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
    unsafe { ClosePrivateNamespace(namespace, 0); }
    
    Ok(success)
}