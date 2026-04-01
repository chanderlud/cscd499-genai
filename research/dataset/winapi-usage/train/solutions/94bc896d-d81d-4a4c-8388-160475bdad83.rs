use windows::core::{Result, HRESULT};
use windows::Win32::Security::Authentication::Identity::DecryptMessage;

fn call_decrypt_message() -> Result<HRESULT> {
    unsafe {
        let result = DecryptMessage(std::ptr::null(), std::ptr::null(), 0, None);

        result.ok()?;

        Ok(result)
    }
}
