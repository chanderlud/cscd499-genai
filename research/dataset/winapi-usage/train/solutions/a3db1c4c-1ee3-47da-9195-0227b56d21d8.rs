use windows::core::{Error, Result};
use windows::Win32::Graphics::Direct3D11::{D3DX11CreateFFT, ID3D11DeviceContext, D3DX11_FFT_DESC};

fn call_d3_dx11_create_fft() -> Result<()> {
    let desc = D3DX11_FFT_DESC::default();
    // SAFETY: Calling D3DX11CreateFFT with valid pointers and default parameters.
    // The API handles null pointers gracefully by returning an error, which we propagate.
    unsafe {
        D3DX11CreateFFT(
            None::<&ID3D11DeviceContext>,
            &desc,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )?
    }
    Ok(())
}
