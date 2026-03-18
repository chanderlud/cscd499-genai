use std::path::Path;
use windows::core::{Error, Result, PCWSTR, PWSTR};
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, MAX_PATH};
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{IShellItem, SHCreateItemFromParsingName, SIGDN_FILESYSPATH};

pub fn shellitem_filesystem_path(path: &Path, output: &mut [u16]) -> Result<usize> {
    // Convert path to wide string with null terminator
    let wide_path = wide_null(path.as_os_str());

    // Validate path length (must fit in MAX_PATH including null)
    if wide_path.len() > MAX_PATH as usize {
        return Err(Error::from_hresult(
            windows::core::HRESULT::from_win32(0x800700CE), // ERROR_FILENAME_EXCED_RANGE
        ));
    }

    // Create IShellItem from parsing name
    let shell_item: IShellItem =
        unsafe { SHCreateItemFromParsingName(PCWSTR(wide_path.as_ptr()), None)? };

    // Get file system path display name
    let display_name: PWSTR = unsafe { shell_item.GetDisplayName(SIGDN_FILESYSPATH)? };

    // Find length of returned string (null-terminated)
    let mut len = 0;
    while unsafe { *display_name.0.add(len) } != 0 {
        len += 1;
    }

    // Check if output buffer is large enough (need space for string + null terminator)
    if len + 1 > output.len() {
        unsafe { CoTaskMemFree(Some(display_name.0 as *const _)) };
        return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
            ERROR_INSUFFICIENT_BUFFER.0,
        )));
    }

    // Copy string to output buffer
    unsafe {
        std::ptr::copy_nonoverlapping(display_name.0, output.as_mut_ptr(), len);
        // Add null terminator
        *output.as_mut_ptr().add(len) = 0;
        // Free the allocated string
        CoTaskMemFree(Some(display_name.0 as *const _));
    }

    Ok(len)
}

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}
