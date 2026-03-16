use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;

pub fn lift_win32<T>(result: std::result::Result<T, WIN32_ERROR>) -> Result<T> {
    result.map_err(|win32_err| Error::from_hresult(win32_err.to_hresult()))
}