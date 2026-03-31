use windows::core::Result;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory2, IDXGIFactory2, DXGI_CREATE_FACTORY_FLAGS,
};

fn call_create_dxgi_factory2() -> WIN32_ERROR {
    let result: Result<IDXGIFactory2> = unsafe { CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0)) };
    match result {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
