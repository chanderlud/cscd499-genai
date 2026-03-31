use windows::core::{Result, BOOL};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::Security::{AccessCheck, GENERIC_MAPPING, PSECURITY_DESCRIPTOR};

fn call_access_check() -> WIN32_ERROR {
    let mut privilegeset_length: u32 = 0;
    let mut granted_access: u32 = 0;
    let mut access_status: BOOL = BOOL(0);
    let generic_mapping = GENERIC_MAPPING::default();

    // SAFETY: We pass null pointers for optional parameters and valid mutable references for output parameters.
    // AccessCheck will fail safely with invalid handles/descriptors, returning an error we catch.
    let result: Result<()> = unsafe {
        AccessCheck(
            PSECURITY_DESCRIPTOR(std::ptr::null_mut()),
            HANDLE(std::ptr::null_mut()),
            0,
            &generic_mapping,
            None,
            &mut privilegeset_length,
            &mut granted_access,
            &mut access_status,
        )
    };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
