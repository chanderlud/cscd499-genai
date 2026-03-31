use windows::core::{Error, Result};
use windows::Win32::Graphics::Direct2D::{D2D1CreateDeviceContext, ID2D1DeviceContext};
use windows::Win32::Graphics::Dxgi::IDXGISurface;

fn call_d2_d1_create_device_context() -> Result<ID2D1DeviceContext> {
    // SAFETY: Calling D2D1CreateDeviceContext with None for surface and properties.
    // The Win32 API safely handles null parameters by returning an appropriate error HRESULT.
    unsafe { D2D1CreateDeviceContext(None::<&IDXGISurface>, None) }
}
