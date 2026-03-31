use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::{AllocateAndInitializeSid, PSID, SID_IDENTIFIER_AUTHORITY};

fn call_allocate_and_initialize_sid() -> WIN32_ERROR {
    let authority = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };
    let mut psid = PSID(std::ptr::null_mut());

    let res: Result<()> =
        unsafe { AllocateAndInitializeSid(&authority, 1, 0, 0, 0, 0, 0, 0, 0, 0, &mut psid) };

    match res {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
