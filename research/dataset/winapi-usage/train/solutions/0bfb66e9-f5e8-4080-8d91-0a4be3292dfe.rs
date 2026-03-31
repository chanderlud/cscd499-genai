use windows::core::HRESULT;
use windows::Win32::Graphics::Direct3D11::{D3DX11CreateFFT, ID3D11DeviceContext};

fn call_d3_dx11_create_fft() -> windows::core::HRESULT {
    unsafe {
        match D3DX11CreateFFT(
            None::<&ID3D11DeviceContext>,
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ) {
            Ok(()) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
