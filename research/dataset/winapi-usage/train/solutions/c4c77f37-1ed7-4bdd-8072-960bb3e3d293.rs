use windows::core::HRESULT;
use windows::Win32::Foundation::S_OK;
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};

fn call_create_dxgi_factory1() -> HRESULT {
    // SAFETY: CreateDXGIFactory1 is a standard Win32 API that safely initializes the DXGI factory.
    match unsafe { CreateDXGIFactory1::<IDXGIFactory1>() } {
        Ok(_) => S_OK,
        Err(e) => e.code(),
    }
}
