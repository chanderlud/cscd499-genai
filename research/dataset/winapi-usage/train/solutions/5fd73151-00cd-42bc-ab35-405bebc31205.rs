#![deny(warnings)]

use windows::core::Result;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Registry::{
    RegCreateKeyExW, HKEY, HKEY_CURRENT_USER, KEY_WRITE, REG_OPTION_NON_VOLATILE,
};

#[allow(dead_code)]
fn call_reg_create_key_ex_w() -> Result<WIN32_ERROR> {
    let mut hkey = HKEY::default();
    // SAFETY: All parameters are valid. `phkresult` points to a valid `HKEY` variable.
    let result = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            windows::core::w!("Software\\TestKey"),
            None,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        )
    };

    result.ok()?;
    Ok(result)
}
