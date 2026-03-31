use windows::core::{Error, Result};
use windows::Win32::Foundation::NTSTATUS;
use windows::Win32::Security::Cryptography::{BCryptCloseAlgorithmProvider, BCRYPT_ALG_HANDLE};

fn call_b_crypt_close_algorithm_provider() -> Result<NTSTATUS> {
    let status =
        unsafe { BCryptCloseAlgorithmProvider(BCRYPT_ALG_HANDLE(std::ptr::null_mut()), 0) };
    status.ok()?;
    Ok(status)
}
