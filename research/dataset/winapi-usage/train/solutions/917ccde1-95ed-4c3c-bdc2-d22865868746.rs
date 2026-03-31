use windows::core::{Error, Result};
use windows::Win32::Security::Authentication::Identity::{
    AcquireCredentialsHandleA, SECPKG_CRED_OUTBOUND,
};
use windows::Win32::Security::Credentials::SecHandle;

fn call_acquire_credentials_handle_a() -> Result<()> {
    let mut credential_handle = SecHandle::default();
    unsafe {
        AcquireCredentialsHandleA(
            None,
            windows::core::s!("Negotiate"),
            SECPKG_CRED_OUTBOUND,
            None,
            None,
            None,
            None,
            &mut credential_handle,
            None,
        )?;
    }
    Ok(())
}
