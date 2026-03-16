use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_MULTI_SZ, REG_OPTION_NON_VOLATILE,
    REG_VALUE_TYPE, RRF_RT_REG_MULTI_SZ, RegCloseKey, RegCreateKeyExW, RegDeleteTreeW,
    RegGetValueW, RegSetValueExW,
};
use windows::core::PCWSTR;

fn wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}

fn create_multi_sz_buffer(items: &[String]) -> Vec<u16> {
    let mut buffer = Vec::new();
    for item in items {
        let wide = wide_null(item);
        buffer.extend_from_slice(&wide);
    }
    // Add final double-null terminator
    buffer.push(0);
    buffer
}

fn decode_multi_sz_buffer(buffer: &[u16]) -> Vec<String> {
    let mut result = Vec::new();
    let mut start = 0;

    for i in 0..buffer.len() {
        if buffer[i] == 0 {
            if i > start {
                let string = String::from_utf16_lossy(&buffer[start..i]);
                result.push(string);
            }
            start = i + 1;

            if i + 1 < buffer.len() && buffer[i + 1] == 0 {
                break;
            }
        }
    }

    result
}

fn validate_multisz_items(items: &[String]) -> std::io::Result<()> {
    for item in items {
        if item.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "REG_MULTI_SZ cannot reliably round-trip empty string elements",
            ));
        }

        if item.encode_utf16().any(|u| u == 0) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "REG_MULTI_SZ strings cannot contain embedded NUL characters",
            ));
        }
    }

    Ok(())
}

struct RegKeyGuard(HKEY);

impl Drop for RegKeyGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

pub fn reg_multisz_roundtrip(
    subkey_path: &str,
    value_name: &str,
    items: &[String],
) -> std::io::Result<Vec<String>> {
    validate_multisz_items(items)?;

    let subkey_wide = wide_null(subkey_path);
    let value_wide = wide_null(value_name);

    let mut hkey = HKEY::default();
    let result = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey_wide.as_ptr()),
            Some(0),
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE | KEY_READ,
            None,
            &mut hkey,
            None,
        )
    };

    if result != ERROR_SUCCESS {
        return Err(std::io::Error::from_raw_os_error(result.0 as i32));
    }

    let key_guard = RegKeyGuard(hkey);

    let multi_sz_data = create_multi_sz_buffer(items);

    let data_bytes = unsafe {
        std::slice::from_raw_parts(
            multi_sz_data.as_ptr() as *const u8,
            multi_sz_data.len() * std::mem::size_of::<u16>(),
        )
    };

    let result = unsafe {
        RegSetValueExW(
            key_guard.0,
            PCWSTR(value_wide.as_ptr()),
            Some(0),
            REG_MULTI_SZ,
            Some(data_bytes),
        )
    };

    if result != ERROR_SUCCESS {
        return Err(std::io::Error::from_raw_os_error(result.0 as i32));
    }

    let mut data_type = REG_VALUE_TYPE::default();
    let mut data_size = 0u32;

    let result = unsafe {
        RegGetValueW(
            key_guard.0,
            PCWSTR::null(),
            PCWSTR(value_wide.as_ptr()),
            RRF_RT_REG_MULTI_SZ,
            Some(&mut data_type),
            None,
            Some(&mut data_size),
        )
    };

    if result != ERROR_SUCCESS {
        return Err(std::io::Error::from_raw_os_error(result.0 as i32));
    }

    let mut buffer = vec![0u16; (data_size as usize) / std::mem::size_of::<u16>()];
    let result = unsafe {
        RegGetValueW(
            key_guard.0,
            PCWSTR::null(),
            PCWSTR(value_wide.as_ptr()),
            RRF_RT_REG_MULTI_SZ,
            Some(&mut data_type),
            Some(buffer.as_mut_ptr() as *mut std::ffi::c_void),
            Some(&mut data_size),
        )
    };

    if result != ERROR_SUCCESS {
        return Err(std::io::Error::from_raw_os_error(result.0 as i32));
    }

    let decoded = decode_multi_sz_buffer(&buffer);

    drop(key_guard);

    let result = unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, PCWSTR(subkey_wide.as_ptr())) };

    if result != ERROR_SUCCESS && result != ERROR_FILE_NOT_FOUND {
        return Err(std::io::Error::from_raw_os_error(result.0 as i32));
    }

    Ok(decoded)
}
