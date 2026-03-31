use std::ffi::c_void;
use windows::core::{Result, PCSTR};
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;
use windows::Win32::Graphics::Direct3D::{ID3DBlob, ID3DInclude};

#[allow(dead_code)]
fn call_d3_d_compile() -> Result<()> {
    let source = b"float4 main() : SV_TARGET { return float4(1.0, 0.0, 0.0, 1.0); }";
    let mut code: Option<ID3DBlob> = None;
    let mut errors: Option<ID3DBlob> = None;

    let entry_point = PCSTR::from_raw(b"main\0".as_ptr());
    let target = PCSTR::from_raw(b"ps_4_0\0".as_ptr());

    unsafe {
        D3DCompile(
            source.as_ptr() as *const c_void,
            source.len(),
            PCSTR::null(),
            None,
            None::<&ID3DInclude>,
            entry_point,
            target,
            0,
            0,
            &mut code,
            Some(&mut errors),
        )?;
    }
    Ok(())
}
