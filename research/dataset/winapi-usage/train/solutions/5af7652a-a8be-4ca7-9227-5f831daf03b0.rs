#![allow(unused_imports)]
use windows::core::{w, Error, Result, HRESULT};
use windows::Win32::System::Registry::{
    RegCreateKeyExW, HKEY, HKEY_CURRENT_USER, KEY_WRITE, REG_OPTION_NON_VOLATILE,
};

#[allow(dead_code)]
fn call_reg_create_key_ex_w() -> HRESULT {
    let mut hkey_result = HKEY::default();
    let err = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\TestKey"),
            None,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey_result,
            None,
        )
    };
    err.to_hresult()
}
