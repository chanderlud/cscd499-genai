use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::Graphics::Imaging::WICCreateBitmapFromSection;

fn call_wic_create_bitmap_from_section() -> WIN32_ERROR {
    // SAFETY: Calling WICCreateBitmapFromSection with concrete, valid parameters.
    // The API is unsafe as it interacts with Win32 handles and pointers.
    match unsafe {
        WICCreateBitmapFromSection(100, 100, std::ptr::null(), HANDLE::default(), 400, 0)
    } {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
