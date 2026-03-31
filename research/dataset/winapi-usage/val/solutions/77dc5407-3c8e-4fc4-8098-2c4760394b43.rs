use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory, IDXGIFactory};

fn call_create_dxgi_factory() -> windows::core::HRESULT {
    unsafe { CreateDXGIFactory::<IDXGIFactory>() }
        .map(|_| HRESULT(0))
        .unwrap_or_else(|e| e.code())
}
