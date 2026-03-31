#![allow(dead_code)]
use windows::core::{Error, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Imaging::{
    GUID_WICPixelFormat32bppBGRA, IWICBitmap, WICCreateBitmapFromSectionEx, WICSectionAccessLevel,
};

fn call_wic_create_bitmap_from_section_ex() -> Result<IWICBitmap> {
    let width = 100u32;
    let height = 100u32;
    let stride = width * 4;
    let offset = 0u32;
    let access_level = WICSectionAccessLevel(0);

    // SAFETY: We pass valid dimensions, a valid pixel format GUID, a null handle (allowed for in-memory allocation),
    // a correctly calculated stride, zero offset, and a valid access level constant.
    unsafe {
        WICCreateBitmapFromSectionEx(
            width,
            height,
            &GUID_WICPixelFormat32bppBGRA,
            HANDLE::default(),
            stride,
            offset,
            access_level,
        )
    }
}
