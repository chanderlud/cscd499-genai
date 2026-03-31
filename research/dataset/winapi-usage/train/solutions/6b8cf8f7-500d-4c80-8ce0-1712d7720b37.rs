#![allow(dead_code)]

use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Direct3D11::{D3DX11CreateFFT, ID3D11DeviceContext};

fn call_d3_dx11_create_fft() -> WIN32_ERROR {
    let result: Result<()> = unsafe {
        D3DX11CreateFFT(
            None::<&ID3D11DeviceContext>,
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
