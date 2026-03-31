use windows::core::{Error, Result};
use windows::Win32::Security::Credentials::{CredDeleteW, CRED_TYPE_GENERIC};

fn call_cred_delete_w() -> Result<()> {
    unsafe {
        CredDeleteW(windows::core::w!("TestTarget"), CRED_TYPE_GENERIC, None)?;
    }
    Ok(())
}
