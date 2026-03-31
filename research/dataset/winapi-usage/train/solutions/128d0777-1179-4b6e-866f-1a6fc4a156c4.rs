#![allow(dead_code)]
use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::Security::{AllocateAndInitializeSid, PSID, SID_IDENTIFIER_AUTHORITY};

fn call_allocate_and_initialize_sid() -> HRESULT {
    let authority = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };
    let mut sid: PSID = PSID(std::ptr::null_mut());
    match unsafe { AllocateAndInitializeSid(&authority, 1, 32, 0, 0, 0, 0, 0, 0, 0, &mut sid) } {
        Ok(()) => HRESULT(0),
        Err(e) => e.code(),
    }
}
