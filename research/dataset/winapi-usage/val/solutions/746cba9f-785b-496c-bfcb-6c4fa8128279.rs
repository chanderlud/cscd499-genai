use windows::core::{Error, Result, HRESULT};
use windows::Win32::Graphics::DirectWrite::{
    DWriteCreateFactory, IDWriteFactory, DWRITE_FACTORY_TYPE_SHARED,
};

fn call_d_write_create_factory() -> HRESULT {
    match unsafe { DWriteCreateFactory::<IDWriteFactory>(DWRITE_FACTORY_TYPE_SHARED) } {
        Ok(_) => HRESULT(0),
        Err(e) => e.code(),
    }
}
