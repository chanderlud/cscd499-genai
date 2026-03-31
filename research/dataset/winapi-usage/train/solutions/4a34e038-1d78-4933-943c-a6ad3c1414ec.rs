use windows::core::{Error, Result};
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile2;
use windows::Win32::Graphics::Direct3D::ID3DBlob;

#[allow(dead_code)]
fn call_d3_d_compile2() -> Result<()> {
    let src = b"float4 main() : SV_TARGET { return float4(1,0,0,1); }\0";
    let mut code_blob: Option<ID3DBlob> = None;
    let mut error_blob: Option<ID3DBlob> = None;

    unsafe {
        D3DCompile2(
            src.as_ptr() as *const _,
            src.len() - 1,
            windows::core::s!("test.hlsl"),
            None,
            None,
            windows::core::s!("main"),
            windows::core::s!("ps_5_0"),
            0,
            0,
            0,
            None,
            0,
            &mut code_blob,
            Some(&mut error_blob),
        )?;
    }
    Ok(())
}
