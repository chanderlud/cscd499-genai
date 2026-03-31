use windows::core::{Result, HRESULT};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory2, IDXGIFactory2, DXGI_CREATE_FACTORY_FLAGS,
};

fn call_create_dxgi_factory2() -> HRESULT {
    let result: Result<IDXGIFactory2> = unsafe { CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0)) };

    match result {
        Ok(_) => HRESULT(0),
        Err(e) => e.code(),
    }
}
