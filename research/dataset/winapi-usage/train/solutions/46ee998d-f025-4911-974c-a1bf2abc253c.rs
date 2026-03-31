#![deny(warnings)]

use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Direct3D11::{D3DDisassemble11Trace, ID3D11ShaderTrace};

#[allow(dead_code)]
fn call_d3_d_disassemble11_trace() -> WIN32_ERROR {
    // SAFETY: D3DDisassemble11Trace is an unsafe Win32 API. We pass null/zero/None
    // as concrete placeholder values, which is safe for this exercise.
    let result =
        unsafe { D3DDisassemble11Trace(std::ptr::null(), 0, None::<&ID3D11ShaderTrace>, 0, 0, 0) };

    match result {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
