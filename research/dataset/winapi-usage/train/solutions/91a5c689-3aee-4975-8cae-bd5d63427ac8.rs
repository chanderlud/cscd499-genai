use windows::core::HRESULT;
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::Storage::Packaging::Appx::{
    AppPolicyGetLifecycleManagement, AppPolicyLifecycleManagement,
};

fn call_app_policy_get_lifecycle_management() -> HRESULT {
    let mut policy = AppPolicyLifecycleManagement(0);
    let result = unsafe { AppPolicyGetLifecycleManagement(HANDLE::default(), &mut policy) };

    result.to_hresult()
}
