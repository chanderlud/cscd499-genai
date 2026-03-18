use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ,
    REG_VALUE_TYPE,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

struct RegistryKeyGuard(HKEY);

impl Drop for RegistryKeyGuard {
    fn drop(&mut self) {
        // SAFETY: FFI call with valid handle
        unsafe { RegCloseKey(self.0) };
    }
}

fn get_registry_string(hive: HKEY, path: &str, value_name: &str) -> Result<String> {
    let wide_path = wide_null(std::ffi::OsStr::new(path));
    let wide_value_name = wide_null(std::ffi::OsStr::new(value_name));

    let mut key_handle = HKEY::default();

    // SAFETY: FFI call with valid parameters
    let result = unsafe {
        RegOpenKeyExW(
            hive,
            PCWSTR(wide_path.as_ptr()),
            Some(0),
            KEY_READ,
            &mut key_handle,
        )
    };

    if result != ERROR_SUCCESS {
        return Err(Error::from_hresult(result.to_hresult()));
    }

    // Ensure key is closed even if subsequent operations fail
    let _guard = RegistryKeyGuard(key_handle);

    let mut data_type = 0u32;
    let mut data_size = 0u32;

    // First call to get required buffer size
    // SAFETY: FFI call with valid parameters, data_size set to 0 to query size
    let result = unsafe {
        RegQueryValueExW(
            key_handle,
            PCWSTR(wide_value_name.as_ptr()),
            None,
            Some(&mut data_type as *mut u32 as *mut REG_VALUE_TYPE),
            None,
            Some(&mut data_size),
        )
    };

    if result != ERROR_SUCCESS {
        return Err(Error::from_hresult(result.to_hresult()));
    }

    if data_type != REG_SZ.0 {
        return Err(Error::from_hresult(ERROR_FILE_NOT_FOUND.to_hresult()));
    }

    // Allocate buffer for the string data
    let mut buffer = vec![0u16; (data_size as usize) / 2];

    // Second call to actually read the data
    // SAFETY: FFI call with valid buffer and size
    let result = unsafe {
        RegQueryValueExW(
            key_handle,
            PCWSTR(wide_value_name.as_ptr()),
            None,
            None,
            Some(buffer.as_mut_ptr() as *mut u8),
            Some(&mut data_size),
        )
    };

    if result != ERROR_SUCCESS {
        return Err(Error::from_hresult(result.to_hresult()));
    }

    // Convert UTF-16 to String, removing null terminator
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    Ok(String::from_utf16_lossy(&buffer[..len]))
}

fn main() -> Result<()> {
    let path = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";
    let value_name = "ProductName";

    match get_registry_string(HKEY_LOCAL_MACHINE, path, value_name) {
        Ok(product_name) => println!("Windows Product Name: {}", product_name),
        Err(e) => {
            eprintln!("Failed to read registry: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
