use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::WIN32_ERROR;

pub fn check_win32(code: WIN32_ERROR) -> Result<()> {
    if code.0 == 0 {
        Ok(())
    } else {
        let hresult = HRESULT::from_win32(code.0);
        Err(Error::from_hresult(hresult))
    }
}