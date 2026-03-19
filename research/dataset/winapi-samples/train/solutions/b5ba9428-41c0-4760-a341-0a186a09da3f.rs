use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{
    ERROR_FILE_NOT_FOUND, ERROR_INSUFFICIENT_BUFFER, ERROR_INVALID_DATA, ERROR_INVALID_NAME,
    ERROR_MORE_DATA, ERROR_SUCCESS,
};
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_SZ,
    REG_VALUE_TYPE,
};

fn wide_null(s: &str) -> Result<[u16; 260]> {
    let mut buffer = [0u16; 260];
    let mut i = 0;
    for c in s.encode_utf16() {
        if i >= 259 {
            return Err(Error::from_hresult(HRESULT::from_win32(
                ERROR_INVALID_NAME.0,
            )));
        }
        buffer[i] = c;
        i += 1;
    }
    buffer[i] = 0;
    Ok(buffer)
}

struct RegKeyGuard(HKEY);

impl Drop for RegKeyGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

fn get_registry_string_noalloc(hive: HKEY, path: &str, value_name: &str) -> Result<String> {
    let path_wide = wide_null(path)?;
    let value_name_wide = wide_null(value_name)?;

    let mut hkey = HKEY::default();
    let result = unsafe {
        RegOpenKeyExW(
            hive,
            PCWSTR(path_wide.as_ptr()),
            Some(0),
            KEY_READ,
            &mut hkey,
        )
    };
    if result != ERROR_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(result.0)));
    }

    let _guard = RegKeyGuard(hkey);

    let mut value_type = REG_VALUE_TYPE::default();
    let mut data_size = 0u32;
    let result = unsafe {
        RegQueryValueExW(
            hkey,
            PCWSTR(value_name_wide.as_ptr()),
            None,
            Some(&mut value_type),
            None,
            Some(&mut data_size),
        )
    };

    if result == ERROR_FILE_NOT_FOUND {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_FILE_NOT_FOUND.0,
        )));
    }
    if result != ERROR_SUCCESS && result != ERROR_MORE_DATA {
        return Err(Error::from_hresult(HRESULT::from_win32(result.0)));
    }

    if value_type != REG_SZ {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INVALID_DATA.0,
        )));
    }

    if data_size > 4096 {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INSUFFICIENT_BUFFER.0,
        )));
    }

    let mut data_buffer = [0u8; 4096];
    let mut actual_size = data_size;
    let result = unsafe {
        RegQueryValueExW(
            hkey,
            PCWSTR(value_name_wide.as_ptr()),
            None,
            Some(&mut value_type),
            Some(data_buffer.as_mut_ptr()),
            Some(&mut actual_size),
        )
    };

    if result != ERROR_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(result.0)));
    }

    let wide_chars = unsafe {
        std::slice::from_raw_parts(
            data_buffer.as_ptr() as *const u16,
            (actual_size as usize) / 2,
        )
    };

    let len = wide_chars
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(wide_chars.len());
    String::from_utf16(&wide_chars[..len])
        .map_err(|_| Error::from_hresult(HRESULT::from_win32(ERROR_INVALID_DATA.0)))
}

pub fn example_usage() -> Result<()> {
    let hive = HKEY_CURRENT_USER;
    let path = "Software\\Test";
    let value_name = "TestValue";
    let result = get_registry_string_noalloc(hive, path, value_name)?;
    println!("Registry value: {}", result);
    Ok(())
}
