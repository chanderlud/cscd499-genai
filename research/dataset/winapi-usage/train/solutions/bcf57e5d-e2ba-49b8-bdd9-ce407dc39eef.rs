use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompile;

fn call_d3_d_compile() -> WIN32_ERROR {
    let result = unsafe {
        D3DCompile(
            std::ptr::null(),
            0,
            None,
            None,
            None,
            None,
            None,
            0,
            0,
            std::ptr::null_mut(),
            None,
        )
    };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
