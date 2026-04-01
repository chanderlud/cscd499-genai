use std::ffi::c_void;
use windows::Win32::System::Variant::{InitVariantFromBuffer, VARIANT};

fn call_init_variant_from_buffer() -> windows::core::Result<windows::core::Result<VARIANT>> {
    let buffer: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

    let result =
        unsafe { InitVariantFromBuffer(buffer.as_ptr() as *const c_void, buffer.len() as u32) };

    Ok(result)
}
