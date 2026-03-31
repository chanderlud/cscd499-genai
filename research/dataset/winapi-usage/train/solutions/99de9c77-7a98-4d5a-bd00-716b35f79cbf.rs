#![deny(warnings)]

use windows::core::HRESULT;
use windows::Win32::Graphics::Imaging::{IWICBitmapSource, WICConvertBitmapSource};

#[allow(dead_code)]
fn call_wic_convert_bitmap_source() -> HRESULT {
    // SAFETY: Passing null pointers is safe; the API will return an error HRESULT
    // which we convert and return.
    unsafe {
        WICConvertBitmapSource(std::ptr::null(), None::<&IWICBitmapSource>)
            .map(|_| HRESULT(0))
            .unwrap_or_else(|e| e.code())
    }
}
