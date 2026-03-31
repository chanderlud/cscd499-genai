use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::Storage::Packaging::Appx::{AppPolicyClrCompat, AppPolicyGetClrCompat};

fn call_app_policy_get_clr_compat() -> Result<WIN32_ERROR> {
    let mut policy = AppPolicyClrCompat(0);
    // SAFETY: AppPolicyGetClrCompat is a Win32 API. We pass a concrete HANDLE value
    // and a valid mutable pointer for the output policy.
    let result = unsafe { AppPolicyGetClrCompat(HANDLE::default(), &mut policy) };
    if result.0 != 0 {
        return Err(Error::from_thread());
    }
    Ok(result)
}
