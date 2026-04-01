use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Variant::InitVariantFromBuffer;

fn call_init_variant_from_buffer() -> WIN32_ERROR {
    let buffer: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

    match unsafe {
        InitVariantFromBuffer(
            buffer.as_ptr() as *const core::ffi::c_void,
            buffer.len() as u32,
        )
    } {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
