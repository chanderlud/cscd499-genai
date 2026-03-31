#![deny(warnings)]

use windows::core::{Error, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Graphics::Gdi::AddFontMemResourceEx;

#[allow(dead_code)]
fn call_add_font_mem_resource_ex() -> Result<HANDLE> {
    let font_data = [0u8; 4];
    let mut num_fonts: u32 = 0;

    // SAFETY: We pass valid pointers and sizes to the Win32 API.
    // Failure is handled by checking for a null handle and capturing GetLastError.
    let handle = unsafe {
        AddFontMemResourceEx(
            font_data.as_ptr() as *const _,
            font_data.len() as u32,
            None,
            &mut num_fonts,
        )
    };

    if handle.0.is_null() {
        return Err(Error::from_thread());
    }

    Ok(handle)
}
