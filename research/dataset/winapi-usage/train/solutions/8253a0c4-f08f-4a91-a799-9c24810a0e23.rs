use windows::core::{Error, Result};
use windows::Win32::Security::Credentials::{CredDeleteA, CRED_TYPE};

fn call_cred_delete_a() -> Result<()> {
    unsafe {
        CredDeleteA(windows::core::s!("TestTarget"), CRED_TYPE(0), None)?;
        Ok(())
    }
}
