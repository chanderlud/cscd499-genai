use windows::core::{Error, Result};
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Graphics::OpenGL::{DescribePixelFormat, PIXELFORMATDESCRIPTOR};

fn call_describe_pixel_format() -> Result<i32> {
    let hdc = HDC::default();
    let mut pfd = PIXELFORMATDESCRIPTOR::default();
    let nbytes = std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u32;

    // SAFETY: We pass a valid mutable pointer to a PIXELFORMATDESCRIPTOR and its correct size.
    // The function returns 0 on failure, which we check and convert to an Error.
    unsafe {
        let result = DescribePixelFormat(hdc, 1, nbytes, Some(&mut pfd));
        if result == 0 {
            return Err(Error::from_thread());
        }
        Ok(result)
    }
}
