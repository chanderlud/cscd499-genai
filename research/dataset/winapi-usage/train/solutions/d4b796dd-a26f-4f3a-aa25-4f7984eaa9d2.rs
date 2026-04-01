use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::Storage::Packaging::Appx::{
    AppPolicyGetLifecycleManagement, AppPolicyLifecycleManagement,
};

fn call_app_policy_get_lifecycle_management() -> Result<AppPolicyLifecycleManagement> {
    let processtoken = HANDLE(std::ptr::null_mut());
    let mut policy = AppPolicyLifecycleManagement(0);

    let result = unsafe { AppPolicyGetLifecycleManagement(processtoken, &mut policy) };

    if result.0 == 0 {
        Ok(policy)
    } else {
        Err(Error::from_hresult(result.to_hresult()))
    }
}
