use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::System::Environment::ExpandEnvironmentStringsW;

pub fn expand_env(input: &str) -> Result<OsString> {
    // Convert input to wide null-terminated string
    let wide_input: Vec<u16> = input.encode_utf16().chain(std::iter::once(0)).collect();

    // First call to get required buffer size
    let required_size = unsafe { ExpandEnvironmentStringsW(PCWSTR(wide_input.as_ptr()), None) };

    if required_size == 0 {
        return Err(Error::from_thread());
    }

    // Allocate buffer with required size
    let mut buffer = vec![0u16; required_size as usize];

    // Second call to expand environment strings
    let result = unsafe {
        ExpandEnvironmentStringsW(PCWSTR(wide_input.as_ptr()), Some(buffer.as_mut_slice()))
    };

    if result == 0 {
        return Err(Error::from_thread());
    }

    // Convert result to OsString (excluding the null terminator)
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    Ok(OsString::from_wide(&buffer[..len]))
}