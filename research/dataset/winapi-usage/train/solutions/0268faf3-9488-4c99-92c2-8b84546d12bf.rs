use windows::core::{Error, Result};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};

#[allow(dead_code)]
fn call_create_dxgi_factory1() -> Result<IDXGIFactory1> {
    unsafe { CreateDXGIFactory1() }
}
