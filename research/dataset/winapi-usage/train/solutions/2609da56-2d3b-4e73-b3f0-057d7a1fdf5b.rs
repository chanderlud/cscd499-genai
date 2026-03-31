use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Direct2D::D2D1CreateDevice;
use windows::Win32::Graphics::Dxgi::IDXGIDevice;

fn call_d2_d1_create_device() -> WIN32_ERROR {
    match unsafe { D2D1CreateDevice(None::<&IDXGIDevice>, None) } {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
