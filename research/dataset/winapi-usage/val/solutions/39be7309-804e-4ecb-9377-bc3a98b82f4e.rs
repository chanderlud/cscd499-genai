use windows::core::{Error, Result};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE_SHARED,
};

fn call_d_write_create_factory() -> Result<IDWriteFactory> {
    unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED) }
}
