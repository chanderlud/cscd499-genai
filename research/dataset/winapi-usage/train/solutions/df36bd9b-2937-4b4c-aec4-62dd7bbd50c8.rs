use windows::core::{Error, Result, PCSTR};
use windows::Win32::Graphics::Gdi::AddFontResourceA;

#[allow(dead_code)]
fn call_add_font_resource_a() -> Result<i32> {
    let font_path = b"test.ttf\0";
    let result = unsafe { AddFontResourceA(PCSTR::from_raw(font_path.as_ptr())) };
    if result == 0 {
        return Err(Error::from_thread());
    }
    Ok(result)
}
