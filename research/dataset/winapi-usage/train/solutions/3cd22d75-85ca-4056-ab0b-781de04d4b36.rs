#![deny(warnings)]

use windows::core::{GUID, HRESULT};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Imaging::WICCreateBitmapFromSection;

#[allow(dead_code)]
fn call_wic_create_bitmap_from_section() -> HRESULT {
    unsafe {
        match WICCreateBitmapFromSection(
            100,
            100,
            core::ptr::null::<GUID>(),
            HANDLE(core::ptr::null_mut()),
            0,
            0,
        ) {
            Ok(_) => HRESULT::default(),
            Err(e) => e.code(),
        }
    }
}
