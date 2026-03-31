#[allow(unused_imports)]
use windows::core::{Error, Result};
use windows::Win32::Graphics::Direct3D::ID3DBlob;
use windows::Win32::Graphics::Direct3D11::{D3DDisassemble11Trace, ID3D11ShaderTrace};

fn call_d3_d_disassemble11_trace() -> Result<ID3DBlob> {
    unsafe {
        D3DDisassemble11Trace(
            std::ptr::null(),
            0,
            Option::<&ID3D11ShaderTrace>::None,
            0,
            0,
            0,
        )
    }
}
