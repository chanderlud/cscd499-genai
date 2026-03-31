#![allow(dead_code)]

use windows::core::w;
use windows::core::{Error, Result};
use windows::Win32::System::Environment::ExpandEnvironmentStringsW;

fn call_expand_environment_strings_w() -> Result<u32> {
    // SAFETY: ExpandEnvironmentStringsW is safe to call with a valid wide string literal and None buffer.
    let result = unsafe { ExpandEnvironmentStringsW(w!("%TEMP%"), None) };
    if result == 0 {
        return Err(Error::from_thread());
    }
    Ok(result)
}
