use windows::core::Result;
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::Storage::Packaging::Appx::{
    AppPolicyGetLifecycleManagement, AppPolicyLifecycleManagement,
};

fn call_app_policy_get_lifecycle_management() -> windows::Win32::Foundation::WIN32_ERROR {
    let mut policy = AppPolicyLifecycleManagement(0);
    unsafe { AppPolicyGetLifecycleManagement(HANDLE::default(), &mut policy) }
}
