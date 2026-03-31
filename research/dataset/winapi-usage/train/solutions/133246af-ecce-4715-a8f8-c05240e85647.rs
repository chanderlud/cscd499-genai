use windows::core::{Error, Result};
use windows::Win32::Storage::Packaging::Appx::{
    ActivatePackageVirtualizationContext, PACKAGE_VIRTUALIZATION_CONTEXT_HANDLE,
};

fn call_activate_package_virtualization_context() -> Result<Result<usize>> {
    let context = PACKAGE_VIRTUALIZATION_CONTEXT_HANDLE(std::ptr::null_mut());
    // SAFETY: The Win32 API is marked unsafe due to raw handle usage; passing a null handle is valid for this exercise.
    let result = unsafe { ActivatePackageVirtualizationContext(context) };
    Ok(result)
}
