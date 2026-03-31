#![allow(dead_code)]

use windows::core::{Error, Result, HRESULT};
use windows::Win32::Graphics::Gdi::HDC;
use windows::Win32::Graphics::OpenGL::{ChoosePixelFormat, PIXELFORMATDESCRIPTOR};

fn call_choose_pixel_format() -> HRESULT {
    unsafe {
        let mut pfd = PIXELFORMATDESCRIPTOR::default();
        pfd.nSize = std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16;
        pfd.nVersion = 1;
        let result = ChoosePixelFormat(HDC(std::ptr::null_mut()), &pfd);
        if result == 0 {
            Error::from_thread().code()
        } else {
            HRESULT::from_win32(0)
        }
    }
}
