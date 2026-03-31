use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Direct2D::D2D1CreateDeviceContext;
use windows::Win32::Graphics::Dxgi::IDXGISurface;

fn call_d2_d1_create_device_context() -> WIN32_ERROR {
    let result = unsafe { D2D1CreateDeviceContext(None::<&IDXGISurface>, None) };
    match result {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_else(|| WIN32_ERROR(e.code().0 as u32)),
    }
}
