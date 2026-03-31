use windows::core::PCWSTR;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::Credentials::{CredEnumerateW, CREDENTIALW};

fn call_cred_enumerate_w() -> WIN32_ERROR {
    let mut count: u32 = 0;
    let mut credential: *mut *mut CREDENTIALW = std::ptr::null_mut();

    let result = unsafe { CredEnumerateW(PCWSTR::null(), None, &mut count, &mut credential) };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
