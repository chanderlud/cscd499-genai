use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_INVALID_DATA};
use windows::Win32::System::Registry::{
    RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ, REG_VALUE_TYPE,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn get_registry_string(hive: HKEY, path: &str, value_name: &str) -> Result<String> {
    let key_path = wide_null(std::ffi::OsStr::new(path));
    let mut hkey = HKEY::default();

    // SAFETY: FFI call to open registry key
    unsafe {
        let result = RegOpenKeyExW(
            hive,
            PCWSTR(key_path.as_ptr()),
            Some(0),
            KEY_READ,
            &mut hkey,
        );
        if result != windows::Win32::Foundation::WIN32_ERROR(0) {
            return Err(Error::from_hresult(HRESULT::from_win32(result.0)));
        }
    }

    let value_name_wide = wide_null(std::ffi::OsStr::new(value_name));
    let mut buffer_size: u32 = 0;
    let mut value_type: REG_VALUE_TYPE = REG_VALUE_TYPE(0);

    // First call to get required buffer size
    // SAFETY: FFI call to query registry value size
    unsafe {
        let result = RegQueryValueExW(
            hkey,
            PCWSTR(value_name_wide.as_ptr()),
            None,
            Some(&mut value_type as *mut REG_VALUE_TYPE),
            None,
            Some(&mut buffer_size),
        );
        if result != windows::Win32::Foundation::WIN32_ERROR(0) {
            return Err(Error::from_hresult(HRESULT::from_win32(result.0)));
        }
    }

    // Check that the value type is REG_SZ
    if value_type != REG_VALUE_TYPE(REG_SZ.0) {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INVALID_DATA.0,
        )));
    }

    // Allocate buffer for the string data
    let mut buffer: Vec<u16> = vec![0; (buffer_size as usize) / 2];

    // Second call to get the actual data
    // SAFETY: FFI call to read registry value data
    unsafe {
        let result = RegQueryValueExW(
            hkey,
            PCWSTR(value_name_wide.as_ptr()),
            None,
            None,
            Some(buffer.as_mut_ptr() as *mut u8),
            Some(&mut buffer_size),
        );
        if result != windows::Win32::Foundation::WIN32_ERROR(0) {
            return Err(Error::from_hresult(HRESULT::from_win32(result.0)));
        }
    }

    // Convert from UTF-16 to Rust String, trimming any null terminators
    let string = String::from_utf16_lossy(&buffer);
    Ok(string.trim_end_matches('\0').to_string())
}

fn get_windows_version() -> Result<String> {
    let path = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";

    // Try to get display version (Windows 10/11)
    match get_registry_string(HKEY_LOCAL_MACHINE, path, "DisplayVersion") {
        Ok(version) => return Ok(version),
        Err(e) if e.code() == HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0) => {
            // Value not found, try fallback
        }
        Err(e) => return Err(e),
    }

    // Fallback to ReleaseId
    match get_registry_string(HKEY_LOCAL_MACHINE, path, "ReleaseId") {
        Ok(release_id) => Ok(release_id),
        Err(e) if e.code() == HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0) => Err(
            Error::from_hresult(HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0)),
        ),
        Err(e) => Err(e),
    }
}

fn main() -> Result<()> {
    match get_windows_version() {
        Ok(version) => println!("Windows version: {}", version),
        Err(e) => eprintln!("Failed to get Windows version: {}", e),
    }
    Ok(())
}
