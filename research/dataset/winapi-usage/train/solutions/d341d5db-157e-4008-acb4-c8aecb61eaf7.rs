use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::Storage::Packaging::Appx::{AppPolicyClrCompat, AppPolicyGetClrCompat};

fn call_app_policy_get_clr_compat() -> WIN32_ERROR {
    let mut policy = AppPolicyClrCompat(0);
    unsafe { AppPolicyGetClrCompat(HANDLE::default(), &mut policy) }
}
