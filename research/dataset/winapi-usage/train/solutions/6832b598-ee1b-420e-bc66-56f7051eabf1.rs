use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Graphics::OpenGL::GetPixelFormat;

fn call_get_pixel_format() -> windows::Win32::Foundation::WIN32_ERROR {
    unsafe {
        let format = GetPixelFormat(HDC::default());
        if format == 0 {
            WIN32_ERROR(Error::from_thread().code().0 as u32)
        } else {
            WIN32_ERROR(0)
        }
    }
}
