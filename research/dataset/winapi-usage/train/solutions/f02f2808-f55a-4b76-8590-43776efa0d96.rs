use windows::core::Error;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Graphics::OpenGL::{ChoosePixelFormat, PIXELFORMATDESCRIPTOR};

fn call_choose_pixel_format() -> WIN32_ERROR {
    unsafe {
        let hdc = HDC::default();
        let pfd = PIXELFORMATDESCRIPTOR::default();
        let result = ChoosePixelFormat(hdc, &pfd);
        if result == 0 {
            WIN32_ERROR(Error::from_thread().code().0 as u32)
        } else {
            WIN32_ERROR(0)
        }
    }
}
