// TITLE: Load and destroy an icon from a file using Win32 API

use std::path::Path;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, LoadImageW, HICON, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn load_icon_from_path(path: &Path) -> Result<HICON> {
    let wide_path = wide_null(path.as_os_str());

    // SAFETY: LoadImageW is a valid Win32 API call with proper parameters
    let handle = unsafe {
        LoadImageW(
            None,
            PCWSTR(wide_path.as_ptr()),
            IMAGE_ICON,
            0, // Use default width
            0, // Use default height
            LR_DEFAULTSIZE | LR_LOADFROMFILE,
        )
    }?;

    // Convert HANDLE to HICON (they're the same underlying type)
    Ok(HICON(handle.0))
}

fn main() -> Result<()> {
    // Example: Load an icon from a file (replace with actual path)
    let icon_path = Path::new("example.ico");

    let icon = load_icon_from_path(icon_path)?;
    println!("Successfully loaded icon from: {}", icon_path.display());

    // SAFETY: DestroyIcon is a valid Win32 API call with a valid icon handle
    unsafe {
        DestroyIcon(icon)?;
    }

    println!("Icon destroyed successfully");
    Ok(())
}
