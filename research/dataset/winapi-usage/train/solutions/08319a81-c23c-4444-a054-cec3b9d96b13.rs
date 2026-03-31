use windows::core::Result;
use windows::Win32::Security::Credentials::{CredEnumerateA, CREDENTIALA, CRED_ENUMERATE_FLAGS};

fn call_cred_enumerate_a() -> Result<()> {
    let mut count: u32 = 0;
    let mut credentials: *mut *mut CREDENTIALA = std::ptr::null_mut();

    // SAFETY: We provide valid mutable pointers for `count` and `credentials`.
    // `CredEnumerateA` will write the count and allocate an array of credential pointers on success.
    unsafe {
        CredEnumerateA(
            windows::core::PCSTR::null(),
            None::<CRED_ENUMERATE_FLAGS>,
            &mut count,
            &mut credentials,
        )?;
    }

    Ok(())
}
