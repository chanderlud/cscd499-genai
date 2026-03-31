use windows::core::{Error, Result};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE_SHARED,
};

fn call_d_write_create_factory() -> windows::Win32::Foundation::WIN32_ERROR {
    // SAFETY: DWriteCreateFactory is an unsafe Win32 API. We pass a valid factory type constant.
    unsafe {
        match DWriteCreateFactory::<IDWriteFactory>(DWRITE_FACTORY_TYPE_SHARED) {
            Ok(_) => windows::Win32::Foundation::WIN32_ERROR(0),
            Err(e) => windows::Win32::Foundation::WIN32_ERROR(e.code().0 as u32),
        }
    }
}
