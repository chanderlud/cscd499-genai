use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegGetValueW, RegSetValueExW, HKEY_CURRENT_USER, KEY_READ,
    KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ, RRF_RT_REG_SZ,
};

fn wide_null(s: &str) -> Vec<u16> {
    use std::iter::once;
    s.encode_utf16().chain(once(0)).collect()
}

pub fn reg_set_get_hkcu(path: &str, name: &str, value: &str) -> Result<String> {
    let path_w = wide_null(path);
    let name_w = wide_null(name);
    let value_w = wide_null(value);

    // Open or create the registry key
    let mut hkey = Default::default();
    let result = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(path_w.as_ptr()),
            Some(0),
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE | KEY_READ,
            None,
            &mut hkey,
            None,
        )
    };
    if result != ERROR_SUCCESS {
        return Err(Error::from_hresult(result.to_hresult()));
    }

    // Set the string value
    let data =
        unsafe { std::slice::from_raw_parts(value_w.as_ptr() as *const u8, value_w.len() * 2) };
    let result =
        unsafe { RegSetValueExW(hkey, PCWSTR(name_w.as_ptr()), Some(0), REG_SZ, Some(data)) };
    if result != ERROR_SUCCESS {
        unsafe { RegCloseKey(hkey) };
        return Err(Error::from_hresult(result.to_hresult()));
    }

    // Read the value back
    let mut buffer_size: u32 = 0;
    let result = unsafe {
        RegGetValueW(
            hkey,
            None,
            PCWSTR(name_w.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            None,
            Some(&mut buffer_size),
        )
    };
    if result != ERROR_SUCCESS {
        unsafe { RegCloseKey(hkey) };
        return Err(Error::from_hresult(result.to_hresult()));
    }

    let mut buffer = vec![0u16; (buffer_size / 2) as usize];
    let result = unsafe {
        RegGetValueW(
            hkey,
            None,
            PCWSTR(name_w.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            Some(buffer.as_mut_ptr() as *mut std::ffi::c_void),
            Some(&mut buffer_size),
        )
    };
    unsafe { RegCloseKey(hkey) };
    if result != ERROR_SUCCESS {
        return Err(Error::from_hresult(result.to_hresult()));
    }

    // Convert the wide string to Rust String
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    Ok(String::from_utf16_lossy(&buffer[..len]))
}