#![deny(warnings)]

#[allow(unused_imports)]
use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Gdi::HENHMETAFILE;
use windows::Win32::Graphics::OpenGL::GetEnhMetaFilePixelFormat;

#[allow(dead_code)]
fn call_get_enh_meta_file_pixel_format() -> WIN32_ERROR {
    // SAFETY: Passing a null handle and zero buffer size is safe and valid for this API.
    let result = unsafe { GetEnhMetaFilePixelFormat(HENHMETAFILE(std::ptr::null_mut()), 0, None) };
    WIN32_ERROR(result)
}
