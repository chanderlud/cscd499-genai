use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::Storage::Packaging::Appx::{
    AppPolicyCreateFileAccess, AppPolicyGetCreateFileAccess,
};

fn call_app_policy_get_create_file_access() -> Result<WIN32_ERROR> {
    let mut policy = AppPolicyCreateFileAccess(0);
    // SAFETY: We pass a valid mutable pointer to `policy` and a default HANDLE.
    let err = unsafe { AppPolicyGetCreateFileAccess(HANDLE::default(), &mut policy) };
    if err.0 != 0 {
        return Err(Error::from_thread());
    }
    Ok(err)
}
