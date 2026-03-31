use windows::core::{w, Result, PCWSTR};
use windows::Win32::Security::Authentication::Identity::{
    AcquireCredentialsHandleW, SECPKG_CRED_OUTBOUND,
};
use windows::Win32::Security::Credentials::SecHandle;

fn call_acquire_credentials_handle_w() -> Result<()> {
    let mut credential_handle: SecHandle = unsafe { std::mem::zeroed() };
    unsafe {
        AcquireCredentialsHandleW(
            PCWSTR::null(),
            w!("Negotiate"),
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
