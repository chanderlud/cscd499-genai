use windows::core::{Error, Result, HRESULT};
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile2;

fn call_d3_d_compile2() -> windows::core::HRESULT {
    let mut ppcode: Option<windows::Win32::Graphics::Direct3D::ID3DBlob> = None;

    // SAFETY: D3DCompile2 is an unsafe FFI function. We pass null pointers, zero sizes,
    // and None for optional parameters, which is safe and valid for this API.
    let result = unsafe {
        D3DCompile2(
            std::ptr::null(),
            0,
            windows::core::PCSTR::null(),
            None,
            None,
            windows::core::PCSTR::null(),
            windows::core::PCSTR::null(),
            0,
            0,
            0,
            None,
            0,
            &mut ppcode as *mut _,
            None,
        )
    };

    result.map(|_| HRESULT(0)).unwrap_or_else(|e| e.code())
}
