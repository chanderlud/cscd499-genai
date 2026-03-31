use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Gdi::AddFontResourceA;

fn call_add_font_resource_a() -> WIN32_ERROR {
    // SAFETY: AddFontResourceA expects a valid null-terminated ANSI string pointer.
    let result = unsafe { AddFontResourceA(windows::core::PCSTR(b"test\0".as_ptr())) };
    if result == 0 {
        let e = Error::from_thread();
        WIN32_ERROR(e.code().0 as u32)
    } else {
        WIN32_ERROR(0)
    }
}
