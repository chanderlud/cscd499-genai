use windows::core::{Error, Result, HRESULT};
use windows::Win32::Graphics::Direct2D::D2D1CreateDevice;
use windows::Win32::Graphics::Dxgi::IDXGIDevice;

fn call_d2_d1_create_device() -> HRESULT {
    unsafe {
        D2D1CreateDevice(None::<&IDXGIDevice>, None)
            .map(|_| HRESULT::default())
            .unwrap_or_else(|e| e.code())
    }
}
