use windows::core::{Error, Result, HRESULT};
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Graphics::OpenGL::GetPixelFormat;

fn call_get_pixel_format() -> HRESULT {
    let hdc = HDC::default();
    let format = unsafe { GetPixelFormat(hdc) };
    if format == 0 {
        Error::from_thread().code()
    } else {
        HRESULT::from_win32(0)
    }
}
