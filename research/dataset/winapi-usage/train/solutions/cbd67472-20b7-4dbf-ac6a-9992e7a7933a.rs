use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Graphics::OpenGL::{SetPixelFormat, PIXELFORMATDESCRIPTOR};

fn call_set_pixel_format() -> Result<WIN32_ERROR> {
    let hdc = HDC(std::ptr::null_mut());
    let format = 1;

    let pfd = PIXELFORMATDESCRIPTOR {
        nSize: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
        nVersion: 1,
        ..Default::default()
    };

    // SetPixelFormat is an unsafe Win32 API call
    unsafe {
        SetPixelFormat(hdc, format, &pfd)?;
        Ok(WIN32_ERROR(0))
    }
}
