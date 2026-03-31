use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::{AccessCheck, PSECURITY_DESCRIPTOR};

fn call_access_check() -> HRESULT {
    match unsafe {
        AccessCheck(
            PSECURITY_DESCRIPTOR(std::ptr::null_mut()),
            HANDLE(std::ptr::null_mut()),
            0,
            std::ptr::null(),
            None,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    } {
        Ok(()) => HRESULT::default(),
        Err(e) => e.code(),
    }
}
