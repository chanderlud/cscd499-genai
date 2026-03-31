#![deny(warnings)]

use windows::core::{Error, HRESULT, PCSTR};
use windows::Win32::Graphics::Gdi::AddFontResourceA;

#[allow(dead_code)]
fn call_add_font_resource_a() -> HRESULT {
    // SAFETY: We pass a valid, statically allocated, null-terminated byte slice.
    let count = unsafe { AddFontResourceA(PCSTR(b"font.ttf\0".as_ptr())) };

    if count == 0 {
        Error::from_thread().code()
    } else {
        HRESULT::from_win32(0)
    }
}
