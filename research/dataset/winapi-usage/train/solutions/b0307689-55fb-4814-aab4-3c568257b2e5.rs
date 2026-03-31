use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::Graphics::Imaging::{WICCreateBitmapFromSectionEx, WICSectionAccessLevel};

fn call_wic_create_bitmap_from_section_ex() -> WIN32_ERROR {
    let result = unsafe {
        WICCreateBitmapFromSectionEx(
            0,
            0,
            std::ptr::null(),
            HANDLE::default(),
            0,
            0,
            WICSectionAccessLevel(0),
        )
    };

    match result {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
