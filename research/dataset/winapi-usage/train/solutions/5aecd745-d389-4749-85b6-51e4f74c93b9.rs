#![deny(warnings)]

use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};

#[allow(dead_code)]
fn call_create_dxgi_factory1() -> WIN32_ERROR {
    match unsafe { CreateDXGIFactory1::<IDXGIFactory1>() } {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
