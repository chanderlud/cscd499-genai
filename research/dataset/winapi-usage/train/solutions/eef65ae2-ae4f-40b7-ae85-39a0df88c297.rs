use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDeviceAndSwapChain, D3D11_CREATE_DEVICE_FLAG,
};

fn call_d3_d11_create_device_and_swap_chain() -> HRESULT {
    unsafe {
        match D3D11CreateDeviceAndSwapChain(
            None,
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
        ) {
            Ok(()) => HRESULT::default(),
            Err(e) => e.code(),
        }
    }
}
