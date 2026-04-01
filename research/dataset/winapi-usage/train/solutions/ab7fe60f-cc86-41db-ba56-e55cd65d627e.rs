use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::PCWSTR;
use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::UI::Shell::{IShellItem, SHCreateItemFromParsingName};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn call_sh_create_item_from_parsing_name() -> WIN32_ERROR {
    let path = wide_null(OsStr::new("C:\\Windows"));

    match unsafe {
        SHCreateItemFromParsingName::<
            PCWSTR,
            Option<&windows::Win32::System::Com::IBindCtx>,
            IShellItem,
        >(
            PCWSTR::from_raw(path.as_ptr()),
            None::<&windows::Win32::System::Com::IBindCtx>,
        )
    } {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(WIN32_ERROR(1)),
    }
}
