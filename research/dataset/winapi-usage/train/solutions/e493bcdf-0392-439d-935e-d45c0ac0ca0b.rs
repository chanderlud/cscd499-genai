use windows::core::Result;
use windows::Win32::Graphics::Direct2D::{D2D1CreateDevice, ID2D1Device};
use windows::Win32::Graphics::Dxgi::IDXGIDevice;

fn call_d2_d1_create_device() -> Result<ID2D1Device> {
    unsafe { D2D1CreateDevice(None::<&IDXGIDevice>, None) }
}
