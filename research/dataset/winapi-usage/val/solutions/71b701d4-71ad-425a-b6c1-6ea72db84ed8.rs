use windows::core::{Error, Result};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory, IDXGIFactory};

fn call_create_dxgi_factory() -> Result<IDXGIFactory> {
    // SAFETY: CreateDXGIFactory is a standard Win32 API that safely creates a DXGI factory.
    unsafe { CreateDXGIFactory::<IDXGIFactory>() }
}
