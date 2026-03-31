use windows::Win32::Foundation::{HMODULE, WIN32_ERROR};
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::{D3D11CreateDevice, D3D11_CREATE_DEVICE_FLAG};

fn call_d3_d11_create_device() -> WIN32_ERROR {
    let result = unsafe {
        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_FLAG(0),
            None,
            7,
            None,
            None,
            None,
        )
    };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
