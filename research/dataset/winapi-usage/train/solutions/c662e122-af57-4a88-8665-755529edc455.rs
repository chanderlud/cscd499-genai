use windows::core::{Error, Result, HRESULT};
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Graphics::OpenGL::DescribePixelFormat;

fn call_describe_pixel_format() -> HRESULT {
    // SAFETY: Calling DescribePixelFormat with a null HDC and zero parameters is safe for querying.
    let result = unsafe { DescribePixelFormat(HDC::default(), 0, 0, None) };
    if result == 0 {
        Error::from_thread().code()
    } else {
        HRESULT::from_win32(0)
    }
}
