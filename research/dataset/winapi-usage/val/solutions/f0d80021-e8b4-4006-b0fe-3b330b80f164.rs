use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::{D3D11CreateDevice, D3D11_CREATE_DEVICE_FLAG};

fn call_d3_d11_create_device() -> HRESULT {
    unsafe {
        match D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE(std::ptr::null_mut()),
            D3D11_CREATE_DEVICE_FLAG(0),
            None,
            7,
            None,
            None,
            None,
        ) {
            Ok(()) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
