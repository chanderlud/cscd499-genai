use std::path::{Path, PathBuf};
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree};
use windows::Win32::UI::Shell::{
    IShellItem, SHCreateItemFromIDList, SHParseDisplayName, SIGDN_FILESYSPATH,
};
use windows::core::{PCWSTR, Result};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn pidl_roundtrip(path: &Path) -> Result<PathBuf> {
    unsafe { _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED) };

    let wide_path = wide_null(path.as_os_str());

    let mut pidl = std::ptr::null_mut();
    let mut attributes = 0u32;

    unsafe {
        SHParseDisplayName(
            PCWSTR(wide_path.as_ptr()),
            None,
            &mut pidl,
            0,
            Some(&mut attributes as *mut u32),
        )?
    };

    let shell_item: IShellItem = unsafe { SHCreateItemFromIDList(pidl) }.map_err(|e| {
        unsafe {
            CoTaskMemFree(Some(pidl as *const _));
        }
        e
    })?;

    let display_name = unsafe { shell_item.GetDisplayName(SIGDN_FILESYSPATH) }.map_err(|e| {
        unsafe {
            CoTaskMemFree(Some(pidl as *const _));
        }
        e
    })?;

    let path_string = unsafe {
        let len = (0..).take_while(|&i| *display_name.0.add(i) != 0).count();
        let slice = std::slice::from_raw_parts(display_name.0, len);
        String::from_utf16_lossy(slice)
    };

    unsafe {
        CoTaskMemFree(Some(pidl as *const _));
        CoTaskMemFree(Some(display_name.0 as *const _));
    }

    Ok(PathBuf::from(path_string))
}
