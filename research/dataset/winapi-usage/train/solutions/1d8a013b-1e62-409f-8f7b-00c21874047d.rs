use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::Registry::{
    RegGetValueW, HKEY_CURRENT_USER, REG_ROUTINE_FLAGS, REG_VALUE_TYPE,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn call_reg_get_value_w() -> Result<WIN32_ERROR> {
    let hkey = HKEY_CURRENT_USER;
    let subkey = wide_null(OsStr::new(r"Software\Microsoft\Windows\CurrentVersion"));
    let value_name = wide_null(OsStr::new("ProgramFilesDir"));

    let mut data_type: REG_VALUE_TYPE = REG_VALUE_TYPE(0);
    let mut data_size: u32 = 0;

    let result = unsafe {
        RegGetValueW(
            hkey,
            PCWSTR::from_raw(subkey.as_ptr()),
            PCWSTR::from_raw(value_name.as_ptr()),
            REG_ROUTINE_FLAGS(0),
            Some(&mut data_type),
            None,
            Some(&mut data_size),
        )
    };

    if result == ERROR_SUCCESS {
        Ok(result)
    } else {
        Err(Error::from_hresult(result.to_hresult()))
    }
}
