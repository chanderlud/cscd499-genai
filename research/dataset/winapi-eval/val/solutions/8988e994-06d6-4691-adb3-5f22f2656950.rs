use std::ffi::OsString;
use std::path::{Path, PathBuf};
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{IShellItem, SHCreateItemFromParsingName, SIGDN_FILESYSPATH};
use windows::core::{PCWSTR, PWSTR, Result};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(once(0)).collect()
}

pub fn shellitem_filesystem_path(path: &Path) -> Result<PathBuf> {
    // Convert path to wide string for Win32 API
    let wide_path = wide_null(path.as_os_str());

    // Create IShellItem from parsing name
    let shell_item: IShellItem = unsafe {
        // SAFETY: SHCreateItemFromParsingName is a COM function that creates an IShellItem
        // from a parsing name. We pass a valid null-terminated wide string.
        SHCreateItemFromParsingName(PCWSTR(wide_path.as_ptr()), None)?
    };

    // Get the filesystem path display name
    let display_name: PWSTR = unsafe {
        // SAFETY: GetDisplayName allocates a string that we must free with CoTaskMemFree
        shell_item.GetDisplayName(SIGDN_FILESYSPATH)?
    };

    // Convert PWSTR to PathBuf
    let result = unsafe {
        // SAFETY: display_name is a valid null-terminated wide string allocated by COM
        let len = (0..).take_while(|&i| *display_name.0.add(i) != 0).count();
        let slice = std::slice::from_raw_parts(display_name.0, len);
        let os_string: OsString = std::os::windows::ffi::OsStringExt::from_wide(slice);
        PathBuf::from(os_string)
    };

    // Free the allocated string
    unsafe {
        // SAFETY: display_name was allocated by COM and must be freed with CoTaskMemFree
        CoTaskMemFree(Some(display_name.0 as *const _));
    }

    Ok(result)
}
