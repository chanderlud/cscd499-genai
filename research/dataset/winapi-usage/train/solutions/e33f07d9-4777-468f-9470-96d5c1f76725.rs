use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Storage::Packaging::Appx::{
    ActivatePackageVirtualizationContext, PACKAGE_VIRTUALIZATION_CONTEXT_HANDLE,
};

fn call_activate_package_virtualization_context() -> WIN32_ERROR {
    let context = PACKAGE_VIRTUALIZATION_CONTEXT_HANDLE(std::ptr::null_mut());
    match unsafe { ActivatePackageVirtualizationContext(context) } {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
