use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory, IDXGIFactory};

fn call_create_dxgi_factory() -> WIN32_ERROR {
    // SAFETY: CreateDXGIFactory is a standard Win32 API that creates a DXGI factory object.
    match unsafe { CreateDXGIFactory::<IDXGIFactory>() } {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(WIN32_ERROR(0)),
    }
}
