use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, HRESULT, PCWSTR};
use windows::Win32::Foundation::ERROR_INVALID_DATA;
use windows::Win32::System::Registry::{
    RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ, REG_VALUE_TYPE,
};

/// Reads a REG_SZ value from the Windows registry.
///
/// # Arguments
/// * `key_path` - The registry key path (e.g., "SOFTWARE\\Microsoft\\Windows\\CurrentVersion")
/// * `value_name` - The name of the value to read (e.g., "ProductName")
///
/// # Returns
/// * `Ok(String)` - The value as a UTF-16 string
/// * `Err(Error)` - If the registry operation fails
fn read_registry_value(key_path: &str, value_name: &str) -> Result<String, Error> {
    // Convert the key path to a null-terminated UTF-16 string
    let key_path_w: Vec<u16> = OsStr::new(key_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Convert the value name to a null-terminated UTF-16 string
    let value_name_w: Vec<u16> = OsStr::new(value_name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Open the registry key with read access
    let mut hkey: HKEY = HKEY(std::ptr::null_mut());
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(key_path_w.as_ptr()),
            None,
            KEY_READ,
            &mut hkey,
        )
    };

    if result != windows::Win32::Foundation::WIN32_ERROR(0) {
        return Err(Error::from_hresult(HRESULT::from_win32(result.0)));
    }

    // First call to get the required buffer size
    let mut data_type: REG_VALUE_TYPE = Default::default();
    let mut data_size: u32 = 0;
    let result = unsafe {
        RegQueryValueExW(
            hkey,
            PCWSTR(value_name_w.as_ptr()),
            None,
            Some(&mut data_type as *mut _),
            None,
            Some(&mut data_size as *mut _),
        )
    };

    if result != windows::Win32::Foundation::WIN32_ERROR(0) {
        return Err(Error::from_hresult(HRESULT::from_win32(result.0)));
    }

    // Verify the value type is REG_SZ
    if data_type != REG_SZ {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INVALID_DATA.0,
        )));
    }

    // Allocate buffer for the value
    let mut buffer = vec![0u16; data_size as usize];

    // Second call to actually read the value
    let result = unsafe {
        RegQueryValueExW(
            hkey,
            PCWSTR(value_name_w.as_ptr()),
            None,
            Some(&mut data_type as *mut _),
            Some(buffer.as_mut_ptr() as *mut u8),
            Some(&mut data_size as *mut _),
        )
    };

    if result != windows::Win32::Foundation::WIN32_ERROR(0) {
        return Err(Error::from_hresult(HRESULT::from_win32(result.0)));
    }

    // Convert from Vec<u16> to String
    let value = String::from_utf16_lossy(&buffer);

    // Close the key (ignore errors as closing a handle should not fail the operation)
    unsafe {
        let _ = windows::Win32::System::Registry::RegCloseKey(hkey);
    }

    Ok(value)
}

fn main() -> Result<(), Error> {
    // Example: Read the Windows Product Name from the registry
    let key_path = r"SOFTWARE\Microsoft\Windows\CurrentVersion";
    let value_name = "ProductName";

    println!("Reading registry value: {}\\{}", key_path, value_name);

    match read_registry_value(key_path, value_name) {
        Ok(value) => {
            println!("Product Name: {}", value);
        }
        Err(e) => {
            eprintln!("Failed to read registry value: {}", e);
            eprintln!("Error code: {}", e.code().0);
        }
    }

    Ok(())
}
