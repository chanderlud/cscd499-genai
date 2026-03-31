use windows::core::HRESULT;
use windows::Win32::Storage::Packaging::Appx::{
    ActivatePackageVirtualizationContext, PACKAGE_VIRTUALIZATION_CONTEXT_HANDLE,
};

fn call_activate_package_virtualization_context() -> HRESULT {
    match unsafe {
        ActivatePackageVirtualizationContext(PACKAGE_VIRTUALIZATION_CONTEXT_HANDLE(
            std::ptr::null_mut(),
        ))
    } {
        Ok(_) => HRESULT(0),
        Err(e) => e.code(),
    }
}
