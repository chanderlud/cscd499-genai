use windows::core::PCSTR;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::Credentials::{CredEnumerateA, CREDENTIALA};

fn call_cred_enumerate_a() -> WIN32_ERROR {
    let mut count: u32 = 0;
    let mut credential: *mut *mut CREDENTIALA = std::ptr::null_mut();

    // SAFETY: CredEnumerateA is an unsafe Win32 API. We provide valid mutable pointers for out parameters.
    let result = unsafe { CredEnumerateA(PCSTR::null(), None, &mut count, &mut credential) };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
