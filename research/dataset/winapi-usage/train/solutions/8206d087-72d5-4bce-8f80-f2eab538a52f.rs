use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::Cryptography::{BCryptCloseAlgorithmProvider, BCRYPT_ALG_HANDLE};

fn call_b_crypt_close_algorithm_provider() -> WIN32_ERROR {
    let handle = BCRYPT_ALG_HANDLE(std::ptr::null_mut());
    let status = unsafe { BCryptCloseAlgorithmProvider(handle, 0) };
    WIN32_ERROR(status.0 as u32)
}
