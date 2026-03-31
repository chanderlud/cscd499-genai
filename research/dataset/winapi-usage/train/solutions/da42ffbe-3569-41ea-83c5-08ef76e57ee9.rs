use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Registry::{RegCloseKey, HKEY_CURRENT_USER};

fn call_reg_close_key() -> Result<WIN32_ERROR> {
    // SAFETY: HKEY_CURRENT_USER is a valid predefined registry key handle.
    let err = unsafe { RegCloseKey(HKEY_CURRENT_USER) };
    if err != WIN32_ERROR(0) {
        return Err(Error::from_hresult(err.to_hresult()));
    }
    Ok(err)
}
