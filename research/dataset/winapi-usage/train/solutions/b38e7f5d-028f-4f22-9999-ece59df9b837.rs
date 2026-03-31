#[allow(unused_imports)]
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Graphics::Dxgi::DXGIDeclareAdapterRemovalSupport;

fn call_dxgi_declare_adapter_removal_support() -> HRESULT {
    match unsafe { DXGIDeclareAdapterRemovalSupport() } {
        Ok(()) => HRESULT::default(),
        Err(e) => e.code(),
    }
}
