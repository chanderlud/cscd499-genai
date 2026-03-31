#![deny(warnings)]

use windows::core::HRESULT;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::Packaging::Appx::{AppPolicyClrCompat, AppPolicyGetClrCompat};

#[allow(dead_code)]
fn call_app_policy_get_clr_compat() -> HRESULT {
    let mut policy = AppPolicyClrCompat(0);
    // SAFETY: AppPolicyGetClrCompat expects a HANDLE and a mutable pointer to AppPolicyClrCompat.
    // We pass a default HANDLE as a concrete value and a valid mutable reference.
    let err = unsafe { AppPolicyGetClrCompat(HANDLE::default(), &mut policy) };
    HRESULT::from_win32(err.0)
}
