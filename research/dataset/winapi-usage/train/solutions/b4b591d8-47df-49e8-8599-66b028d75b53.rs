#![deny(warnings)]

use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Imaging::{IWICBitmapSource, WICConvertBitmapSource};

#[allow(dead_code)]
fn call_wic_convert_bitmap_source() -> WIN32_ERROR {
    // SAFETY: Passing null/None to WICConvertBitmapSource is safe for this exercise;
    // the API will return an error HRESULT which we convert to WIN32_ERROR.
    match unsafe { WICConvertBitmapSource(std::ptr::null(), None::<&IWICBitmapSource>) } {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
