use windows::core::{Error, Result};
use windows::Win32::Graphics::Direct3D11::{D3DDisassemble11Trace, ID3D11ShaderTrace};

fn call_d3_d_disassemble11_trace() -> windows::core::HRESULT {
    unsafe {
        D3DDisassemble11Trace(std::ptr::null(), 0, None::<&ID3D11ShaderTrace>, 0, 0, 0)
            .map(|_| windows::core::HRESULT(0))
            .unwrap_or_else(|e| e.code())
    }
}
