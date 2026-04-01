use core::ffi::c_void;
use windows::core::{Error, Result};
use windows::Win32::System::Registry::{
    RegGetValueW, HKEY_CURRENT_USER, REG_ROUTINE_FLAGS, REG_VALUE_TYPE,
};

fn call_reg_get_value_w() -> Result<()> {
    let hkey = HKEY_CURRENT_USER;

    let subkey_str = wide_null(&std::ffi::OsStr::new(
        r"Software\Microsoft\Windows\CurrentVersion\Run",
    ));
    let value_name_str = wide_null(&std::ffi::OsStr::new("TestValue"));

    let subkey = windows::core::PCWSTR::from_raw(subkey_str.as_ptr());
    let value_name = windows::core::PCWSTR::from_raw(value_name_str.as_ptr());

    let result = unsafe {
        RegGetValueW(
            hkey,
            subkey,
            value_name,
            REG_ROUTINE_FLAGS(0),
            None::<*mut REG_VALUE_TYPE>,
            None::<*mut c_void>,
            None::<*mut u32>,
        )
    };

    result.ok()
}

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(std::iter::once(0)).collect()
}
