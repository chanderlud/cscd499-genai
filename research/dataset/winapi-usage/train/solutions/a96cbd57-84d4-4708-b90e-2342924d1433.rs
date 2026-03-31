#![deny(warnings)]

#[allow(unused_imports)]
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Graphics::Direct2D::D2D1CreateDeviceContext;
use windows::Win32::Graphics::Dxgi::IDXGISurface;

#[allow(dead_code)]
fn call_d2_d1_create_device_context() -> HRESULT {
    // SAFETY: Passing None for parameters is safe for this exercise.
    // The API will return an error HRESULT which we capture and return.
    unsafe {
        match D2D1CreateDeviceContext(None::<&IDXGISurface>, None) {
            Ok(_) => HRESULT::from_win32(0),
            Err(e) => e.code(),
        }
    }
}
