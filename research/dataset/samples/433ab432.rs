// TITLE: Display a message box using MessageBoxW

use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_OK};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn main() {
    let message = wide_null(std::ffi::OsStr::new("Hello from Windows API!"));
    let title = wide_null(std::ffi::OsStr::new("Example"));

    unsafe {
        MessageBoxW(
            Some(HWND(std::ptr::null_mut())),
            PCWSTR(message.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK,
        );
    }
}
