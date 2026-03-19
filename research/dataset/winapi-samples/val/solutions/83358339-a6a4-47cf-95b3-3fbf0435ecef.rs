use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, HRESULT, PCWSTR, PWSTR};
use windows::Win32::Foundation::{
    ERROR_BAD_PATHNAME, ERROR_INVALID_DATA, ERROR_MORE_DATA, ERROR_SUCCESS,
};
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, KEY_READ, REG_SZ,
};
use windows::Win32::UI::Shell::{IShellItem, SHCreateItemFromParsingName, SIGDN_FILESYSPATH};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn copy_wide_to_buffer(src: &[u16], dst: &mut [u16]) -> Result<usize> {
    let len = src.len();
    if len == 0 {
        return Ok(0);
    }

    // Find the null terminator in src
    let src_len = src.iter().position(|&c| c == 0).unwrap_or(len);

    // Check if dst has enough space (including null terminator)
    if src_len + 1 > dst.len() {
        return Err(Error::from_hresult(HRESULT::from_win32(ERROR_MORE_DATA.0)));
    }

    // Copy the string (including null terminator) without heap allocation
    dst[..src_len + 1].copy_from_slice(&src[..src_len + 1]);

    Ok(src_len)
}

pub fn registry_path_to_shellitem_path(
    hive: HKEY,
    key_path: &str,
    value_name: &str,
    output: &mut [u16],
) -> Result<usize> {
    // Convert strings to wide for Win32 APIs
    let wide_key_path = wide_null(OsStr::new(key_path));
    let wide_value_name = wide_null(OsStr::new(value_name));

    // Open the registry key
    let mut hkey = HKEY::default();
    // SAFETY: FFI call with valid parameters. hkey is an out parameter.
    let status = unsafe {
        RegOpenKeyExW(
            hive,
            PCWSTR(wide_key_path.as_ptr()),
            Some(0),
            KEY_READ,
            &mut hkey,
        )
    };

    if status != ERROR_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0)));
    }

    // Ensure we close the key when done
    // SAFETY: hkey is a valid handle from RegOpenKeyExW
    let hkey_guard = hkey;

    // First query to get required buffer size
    let mut data_type = REG_SZ;
    let mut data_size = 0u32;

    // SAFETY: FFI call with valid parameters. data_size is an out parameter.
    let status = unsafe {
        RegQueryValueExW(
            hkey_guard,
            PCWSTR(wide_value_name.as_ptr()),
            None,
            Some(&mut data_type),
            None,
            Some(&mut data_size),
        )
    };

    if status != ERROR_SUCCESS {
        // SAFETY: hkey_guard is a valid handle
        unsafe {
            RegCloseKey(hkey_guard);
        }
        return Err(Error::from_hresult(HRESULT::from_win32(status.0)));
    }

    // Verify it's a REG_SZ value
    if data_type != REG_SZ {
        // SAFETY: hkey_guard is a valid handle
        unsafe {
            RegCloseKey(hkey_guard);
        }
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INVALID_DATA.0,
        )));
    }

    // Allocate buffer for the registry data
    let mut wide_buf = vec![0u16; (data_size as usize) / 2 + 1];

    // Second query to get the actual data
    // SAFETY: FFI call with valid parameters. wide_buf is properly sized.
    let status = unsafe {
        RegQueryValueExW(
            hkey_guard,
            PCWSTR(wide_value_name.as_ptr()),
            None,
            Some(&mut data_type),
            Some(wide_buf.as_mut_ptr() as *mut u8),
            Some(&mut data_size),
        )
    };

    // SAFETY: hkey_guard is a valid handle
    unsafe {
        RegCloseKey(hkey_guard);
    }

    if status != ERROR_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(status.0)));
    }

    // Find the actual length of the wide string (up to null terminator)
    let path_len = wide_buf
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(wide_buf.len());

    // Validate path length (including null terminator) <= 260
    if path_len + 1 > 260 {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_BAD_PATHNAME.0,
        )));
    }

    // Check if path is absolute by examining wide characters
    let is_absolute = if path_len >= 3 {
        // Check for drive letter (C:\)
        let is_drive = wide_buf[0] >= 'A' as u16
            && wide_buf[0] <= 'Z' as u16
            && wide_buf[1] == ':' as u16
            && wide_buf[2] == '\\' as u16;

        // Check for UNC path (\\)
        let is_unc = wide_buf[0] == '\\' as u16 && wide_buf[1] == '\\' as u16;

        is_drive || is_unc
    } else {
        false
    };

    if !is_absolute {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_BAD_PATHNAME.0,
        )));
    }

    // Create IShellItem from the path
    // SAFETY: FFI call with valid wide string path
    let shell_item: IShellItem =
        unsafe { SHCreateItemFromParsingName(PCWSTR(wide_buf.as_ptr()), None)? };

    // Get the normalized file system path
    // SAFETY: FFI call with valid shell_item
    let display_name = unsafe { shell_item.GetDisplayName(SIGDN_FILESYSPATH)? };

    // Ensure we free the display name when done
    let display_name_guard = display_name;

    // Find the length of the returned wide string
    let mut display_len = 0;
    // SAFETY: display_name is a valid wide string from GetDisplayName
    while unsafe { *display_name_guard.0.add(display_len) } != 0 {
        display_len += 1;
    }

    // Copy to output buffer without heap allocation
    // SAFETY: display_name is valid for display_len elements
    let display_slice = unsafe {
        std::slice::from_raw_parts(display_name_guard.0, display_len + 1) // +1 for null terminator
    };

    let result = copy_wide_to_buffer(display_slice, output);

    // SAFETY: display_name_guard is a valid PWSTR from GetDisplayName
    if !display_name_guard.is_null() {
        unsafe {
            CoTaskMemFree(Some(display_name_guard.0 as *const _));
        }
    }

    result
}
