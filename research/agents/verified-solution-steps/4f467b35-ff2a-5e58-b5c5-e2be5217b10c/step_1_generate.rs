#[allow(unused_imports)]
use windows::core::{Result, Error};
use windows::core::HRESULT;
use windows::Win32::Foundation::WIN32_ERROR;

pub fn win32_to_hresult(code: WIN32_ERROR) -> HRESULT {
    HRESULT::from_win32(code.0)
}