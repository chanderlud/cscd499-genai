use windows::core::HRESULT;
use windows::core::PCWSTR;
use windows::Win32::Foundation::S_OK;
use windows::Win32::System::Registry::{RegGetValueW, HKEY_CURRENT_USER, REG_ROUTINE_FLAGS};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn call_reg_get_value_w() -> windows::core::HRESULT {
    unsafe {
        let subkey = wide_null(std::ffi::OsStr::new(r"Software\MyApp"));
        let value_name = wide_null(std::ffi::OsStr::new("TestValue"));

        let result = RegGetValueW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            PCWSTR(value_name.as_ptr()),
            REG_ROUTINE_FLAGS(0),
            None,
            None,
            None,
        );

        if result.is_ok() {
            S_OK
        } else {
            HRESULT::from_win32(result.0)
        }
    }
}
