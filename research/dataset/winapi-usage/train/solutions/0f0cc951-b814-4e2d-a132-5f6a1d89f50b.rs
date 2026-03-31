use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Graphics::OpenGL::{DescribePixelFormat, PIXELFORMATDESCRIPTOR};

#[allow(dead_code)]
fn call_describe_pixel_format() -> WIN32_ERROR {
    let hdc = HDC(std::ptr::null_mut());
    let nbytes = std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u32;

    let result = unsafe { DescribePixelFormat(hdc, 1, nbytes, None) };

    WIN32_ERROR(result as u32)
}
