use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Imaging::{WICCreateBitmapFromSectionEx, WICSectionAccessLevel};

fn call_wic_create_bitmap_from_section_ex() -> HRESULT {
    unsafe {
        match WICCreateBitmapFromSectionEx(
            100,
            100,
            std::ptr::null(),
            HANDLE::default(),
            0,
            0,
            WICSectionAccessLevel(0),
        ) {
            Ok(_) => HRESULT::default(),
            Err(e) => e.code(),
        }
    }
}
