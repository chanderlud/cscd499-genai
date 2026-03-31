use windows::core::{Result, PCSTR};
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;

fn call_d3_d_compile() -> Result<()> {
    unsafe {
        D3DCompile(
            std::ptr::null(),
            0,
            PCSTR::null(),
            None,
            None::<&windows::Win32::Graphics::Direct3D::ID3DInclude>,
            PCSTR::null(),
            PCSTR::null(),
            0,
            0,
            std::ptr::null_mut(),
            None,
        )
    }
}
