use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::Win32::Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, RegCloseKey, RegEnumValueW, RegOpenKeyExW,
};
use windows::core::{Error, HRESULT, PCWSTR, PWSTR, Result};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

pub fn reg_list_values_hkcu(path: &str) -> Result<Vec<String>> {
    // Handle empty path case
    if path.is_empty() {
        return Err(Error::from_hresult(HRESULT::from_win32(87))); // ERROR_INVALID_PARAMETER
    }

    let wide_path = wide_null(OsStr::new(path));
    let mut hkey = HKEY::default();

    // Open the registry key
    let result = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(wide_path.as_ptr()),
            None,
            KEY_READ,
            &mut hkey,
        )
    };

    if result != ERROR_SUCCESS {
        return Err(Error::from_hresult(result.to_hresult()));
    }

    // Ensure we close the key handle when done
    struct RegKeyGuard(HKEY);
    impl Drop for RegKeyGuard {
        fn drop(&mut self) {
            unsafe {
                let _ = RegCloseKey(self.0);
            }
        }
    }
    let _guard = RegKeyGuard(hkey);

    let mut names = Vec::new();
    let mut index = 0u32;

    loop {
        let mut name_buffer = [0u16; 256];
        let mut name_len = name_buffer.len() as u32;
        let mut value_type = 0u32;

        let result = unsafe {
            RegEnumValueW(
                hkey,
                index,
                Some(PWSTR(name_buffer.as_mut_ptr())),
                &mut name_len,
                None,
                Some(&mut value_type),
                None,
                None,
            )
        };

        match result {
            ERROR_SUCCESS => {
                // Convert the wide string to Rust String
                let name = String::from_utf16_lossy(&name_buffer[..name_len as usize]);
                names.push(name);
                index += 1;
            }
            ERROR_NO_MORE_ITEMS => break,
            err => {
                return Err(Error::from_hresult(err.to_hresult()));
            }
        }
    }

    names.sort();
    Ok(names)
}
