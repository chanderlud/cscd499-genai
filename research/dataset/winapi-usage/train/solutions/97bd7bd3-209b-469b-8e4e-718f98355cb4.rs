use windows::core::{Error, Result};
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Graphics::OpenGL::GetPixelFormat;

fn call_get_pixel_format() -> Result<i32> {
    let hdc = HDC::default();
    let format = unsafe { GetPixelFormat(hdc) };
    if format == 0 {
        Err(Error::from_thread())
    } else {
        Ok(format)
    }
}
