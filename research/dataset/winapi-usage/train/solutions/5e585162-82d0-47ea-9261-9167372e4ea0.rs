use windows::core::{Error, Result};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory2, IDXGIFactory2, DXGI_CREATE_FACTORY_FLAGS,
};

fn call_create_dxgi_factory2() -> Result<IDXGIFactory2> {
    // SAFETY: CreateDXGIFactory2 is an unsafe Win32 API. We provide a valid interface type
    // and zero flags, which is safe and standard for creating a DXGI factory.
    unsafe { CreateDXGIFactory2::<IDXGIFactory2>(DXGI_CREATE_FACTORY_FLAGS(0)) }
}
