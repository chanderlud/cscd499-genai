use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Registry::{RegCloseKey, HKEY};

#[allow(dead_code)]
fn call_reg_close_key() -> WIN32_ERROR {
    unsafe { RegCloseKey(HKEY::default()) }
}
