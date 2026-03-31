use windows::core::Result;
use windows::Win32::Foundation::{HMODULE, WIN32_ERROR};
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDeviceAndSwapChain, D3D11_CREATE_DEVICE_FLAG,
};

fn call_d3_d11_create_device_and_swap_chain() -> WIN32_ERROR {
    // SAFETY: D3D11CreateDeviceAndSwapChain is an unsafe FFI function. We pass None for all
    // optional pointer parameters and valid default values for the rest, satisfying its contract.
    let result: Result<()> = unsafe {
        D3D11CreateDeviceAndSwapChain(
            None::<&windows::Win32::Graphics::Dxgi::IDXGIAdapter>,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_FLAG(0),
            None,
            7,
            None,
            None,
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
