use windows::core::{Result, PCWSTR};
use windows::Win32::Security::Credentials::{CredEnumerateW, CREDENTIALW};

fn call_cred_enumerate_w() -> Result<()> {
    let mut count = 0u32;
    let mut credentials: *mut *mut CREDENTIALW = std::ptr::null_mut();

    // SAFETY: We provide valid mutable pointers for `count` and `credentials`.
    // The API allocates memory for the credential array on success; we intentionally
    // omit freeing it here to keep the example focused on the API call itself.
    unsafe {
        CredEnumerateW(PCWSTR::null(), None, &mut count, &mut credentials)?;
    }

    Ok(())
}
