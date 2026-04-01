use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Result, PCWSTR};
use windows::Win32::UI::Shell::{IShellItem, SHCreateItemFromParsingName};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn call_sh_create_item_from_parsing_name() -> Result<IShellItem> {
    let path = wide_null(OsStr::new("C:\\"));

    unsafe { SHCreateItemFromParsingName(PCWSTR::from_raw(path.as_ptr()), None) }
}
